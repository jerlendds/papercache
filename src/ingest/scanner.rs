use serde::Serialize;
use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};
use walkdir::WalkDir;

use crate::{
    app_state::{JobPayload, JobSignal},
    db::{
        documents::{self, UpsertDocument},
        folders,
        jobs::enqueue_jobs,
        models::JobKind,
    },
    util::{
        fs::{is_skipped_dir, is_supported_document},
        time::system_time_rfc3339,
    },
    web::events::AppEvent,
};

#[derive(Debug, Serialize)]
pub struct ScanSummary {
    pub folder_id: String,
    pub discovered: usize,
    pub missing_marked: usize,
}

pub async fn scan_folder(
    db: &SqlitePool,
    job_signal_tx: &mpsc::Sender<JobSignal>,
    event_tx: &broadcast::Sender<AppEvent>,
    folder_id: &str,
) -> anyhow::Result<ScanSummary> {
    let folder = folders::get_folder(db, folder_id).await?;
    if folder.enabled == 0 {
        return Ok(ScanSummary {
            folder_id: folder_id.to_string(),
            discovered: 0,
            missing_marked: 0,
        });
    }

    let root = std::path::PathBuf::from(&folder.path);
    let mut seen_paths = Vec::new();
    let mut pending = Vec::new();
    let max_depth = if folder.recursive == 1 { usize::MAX } else { 1 };
    let walker = WalkDir::new(&root)
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|entry| !entry.file_type().is_dir() || !is_skipped_dir(entry.path()));

    for entry in walker {
        let entry = entry?;
        if !entry.file_type().is_file() || !is_supported_document(entry.path()) {
            continue;
        }

        let canonical = entry.path().canonicalize()?;
        let metadata = std::fs::metadata(&canonical)?;
        let modified_at = metadata
            .modified()
            .map(system_time_rfc3339)
            .unwrap_or_else(|_| crate::util::time::now_rfc3339());
        let path = canonical.to_string_lossy().to_string();
        let file_name = canonical
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document")
            .to_string();
        pending.push(UpsertDocument {
            folder_id: folder_id.to_string(),
            path: path.clone(),
            canonical_path: path,
            file_name,
            file_size: metadata.len() as i64,
            modified_at,
        });

        if pending.len() == 10 {
            flush_discovered_batch(
                db,
                job_signal_tx,
                event_tx,
                folder_id,
                &mut pending,
                &mut seen_paths,
            )
            .await?;
        }
    }
    flush_discovered_batch(
        db,
        job_signal_tx,
        event_tx,
        folder_id,
        &mut pending,
        &mut seen_paths,
    )
    .await?;

    let before = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM documents WHERE folder_id = ? AND status != 'missing'",
    )
    .bind(folder_id)
    .fetch_one(db)
    .await
    .unwrap_or(0);
    documents::mark_folder_missing_except(db, folder_id, &seen_paths).await?;
    folders::touch_scan(db, folder_id).await?;
    let _ = event_tx.send(AppEvent::FolderScanCompleted {
        folder_id: folder_id.to_string(),
        discovered: seen_paths.len(),
    });

    Ok(ScanSummary {
        folder_id: folder_id.to_string(),
        discovered: seen_paths.len(),
        missing_marked: before.saturating_sub(seen_paths.len() as i64) as usize,
    })
}

async fn flush_discovered_batch(
    db: &SqlitePool,
    job_signal_tx: &mpsc::Sender<JobSignal>,
    event_tx: &broadcast::Sender<AppEvent>,
    folder_id: &str,
    pending: &mut Vec<UpsertDocument>,
    seen_paths: &mut Vec<String>,
) -> anyhow::Result<()> {
    if pending.is_empty() {
        return Ok(());
    }

    let inputs = std::mem::take(pending);
    let document_ids = documents::upsert_discovered_batch(db, &inputs).await?;
    seen_paths.extend(inputs.iter().map(|input| input.path.clone()));

    for document_id in &document_ids {
        let _ = event_tx.send(AppEvent::DocumentDiscovered {
            document_id: document_id.clone(),
            folder_id: folder_id.to_string(),
        });
    }

    let mut ingest_payloads = Vec::new();
    for document_id in document_ids {
        let status = sqlx::query_scalar::<_, String>("SELECT status FROM documents WHERE id = ?")
            .bind(&document_id)
            .fetch_one(db)
            .await?;
        if status != "failed" {
            ingest_payloads.push(JobPayload::ingest_pdf(document_id));
        }
    }

    enqueue_jobs(db, job_signal_tx, JobKind::IngestPdf, ingest_payloads, 0).await?;
    Ok(())
}
