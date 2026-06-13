use std::{collections::HashMap, path::PathBuf, time::Duration};

use notify::{EventKind, RecursiveMode, Watcher};
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};

use crate::{
    app_state::{JobPayload, JobSignal},
    config::WATCH_DEBOUNCE,
    db::{documents, folders, jobs::enqueue_job, models::JobKind},
    index::commands::IndexCommand,
    util::fs::is_pdf,
    web::events::AppEvent,
};

pub async fn run_manager(
    db: SqlitePool,
    job_signal_tx: mpsc::Sender<JobSignal>,
    event_tx: broadcast::Sender<AppEvent>,
    tantivy_tx: mpsc::Sender<IndexCommand>,
) {
    loop {
        if let Err(error) = watch_once(
            db.clone(),
            job_signal_tx.clone(),
            event_tx.clone(),
            tantivy_tx.clone(),
        )
        .await
        {
            tracing::warn!(?error, "folder watcher stopped; restarting soon");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn watch_once(
    db: SqlitePool,
    job_signal_tx: mpsc::Sender<JobSignal>,
    _event_tx: broadcast::Sender<AppEvent>,
    tantivy_tx: mpsc::Sender<IndexCommand>,
) -> anyhow::Result<()> {
    let folders = folders::list_enabled(&db).await?;
    let (raw_tx, mut raw_rx) = mpsc::channel(256);
    let mut watcher = notify::recommended_watcher(move |result| {
        let _ = raw_tx.blocking_send(result);
    })?;

    for folder in &folders {
        let mode = if folder.recursive == 1 {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher.watch(std::path::Path::new(&folder.path), mode)?;
    }

    let mut pending: HashMap<PathBuf, tokio::task::JoinHandle<()>> = HashMap::new();
    while let Some(event) = raw_rx.recv().await {
        let event = event?;
        for path in event.paths {
            if !is_pdf(&path) {
                continue;
            }
            if matches!(event.kind, EventKind::Remove(_)) {
                if let Ok(Some(document_id)) = documents::mark_missing_by_path(&db, &path).await {
                    let _ = tantivy_tx
                        .send(IndexCommand::DeleteDocument { document_id })
                        .await;
                }
                continue;
            }

            if let Some(handle) = pending.remove(&path) {
                handle.abort();
            }
            let db = db.clone();
            let tx = job_signal_tx.clone();
            let path_clone = path.clone();
            pending.insert(
                path,
                tokio::spawn(async move {
                    tokio::time::sleep(WATCH_DEBOUNCE).await;
                    if file_stable(&path_clone).await {
                        if let Ok(Some(document)) = documents::find_by_path(&db, &path_clone).await
                        {
                            let _ = enqueue_job(
                                &db,
                                &tx,
                                JobKind::IngestPdf,
                                JobPayload::ingest_pdf(document.id),
                                5,
                            )
                            .await;
                        } else if let Ok(folders) = folders::list_enabled(&db).await {
                            if let Some(folder) = folders
                                .into_iter()
                                .find(|folder| path_clone.starts_with(&folder.path))
                            {
                                let _ = enqueue_job(
                                    &db,
                                    &tx,
                                    JobKind::ScanFolder,
                                    JobPayload::scan_folder(folder.id),
                                    5,
                                )
                                .await;
                            }
                        }
                    }
                }),
            );
        }
    }
    Ok(())
}

async fn file_stable(path: &PathBuf) -> bool {
    let first = tokio::fs::metadata(path).await.ok();
    tokio::time::sleep(Duration::from_secs(1)).await;
    let second = tokio::fs::metadata(path).await.ok();
    match (first, second) {
        (Some(a), Some(b)) => a.len() == b.len() && a.modified().ok() == b.modified().ok(),
        _ => false,
    }
}
