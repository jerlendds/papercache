use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};

use crate::{
    app_state::{JobPayload, JobSignal},
    db::{assets::AssetStore, jobs, models::Job},
    index::commands::IndexCommand,
    ingest::{pipeline, scanner},
    web::events::AppEvent,
};

pub async fn run_workers(
    db: SqlitePool,
    mut job_signal_rx: mpsc::Receiver<JobSignal>,
    job_signal_tx: mpsc::Sender<JobSignal>,
    event_tx: broadcast::Sender<AppEvent>,
    tantivy_tx: mpsc::Sender<IndexCommand>,
    asset_store: std::sync::Arc<AssetStore>,
) {
    loop {
        drain_jobs(
            &db,
            &job_signal_tx,
            &event_tx,
            &tantivy_tx,
            asset_store.as_ref(),
        )
        .await;

        tokio::select! {
            Some(_) = job_signal_rx.recv() => {}
            _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {}
        }
    }
}

async fn drain_jobs(
    db: &SqlitePool,
    job_signal_tx: &mpsc::Sender<JobSignal>,
    event_tx: &broadcast::Sender<AppEvent>,
    tantivy_tx: &mpsc::Sender<IndexCommand>,
    asset_store: &AssetStore,
) {
    loop {
        let claimed = match jobs::claim_next_job(db).await {
            Ok(job) => job,
            Err(error) => {
                tracing::error!(?error, "failed to claim job");
                return;
            }
        };
        let Some(job) = claimed else {
            return;
        };

        if let Err(error) = jobs::mark_started(db, event_tx, &job).await {
            tracing::warn!(?error, job_id = %job.id, "failed to mark job started");
        }

        let result = run_job(db, job_signal_tx, event_tx, tantivy_tx, asset_store, &job).await;
        match result {
            Ok(()) => {
                if let Err(error) = jobs::mark_succeeded(db, &job).await {
                    tracing::error!(?error, job_id = %job.id, "failed to mark job succeeded");
                }
            }
            Err(error) => {
                let message = error.to_string();
                if let Err(mark_error) = jobs::mark_failed_or_retry(
                    db,
                    job_signal_tx,
                    event_tx,
                    &job,
                    &message,
                    is_permanent_job_error(&message),
                )
                .await
                {
                    tracing::error!(?mark_error, job_id = %job.id, "failed to update failed job");
                }
            }
        }
    }
}

async fn run_job(
    db: &SqlitePool,
    job_signal_tx: &mpsc::Sender<JobSignal>,
    event_tx: &broadcast::Sender<AppEvent>,
    tantivy_tx: &mpsc::Sender<IndexCommand>,
    asset_store: &AssetStore,
    job: &Job,
) -> anyhow::Result<()> {
    let payload: JobPayload = serde_json::from_str(&job.payload_json)?;
    match job.kind.as_str() {
        "scan_folder" => {
            let folder_id = payload
                .folder_id
                .ok_or_else(|| anyhow::anyhow!("scan_folder missing folder_id"))?;
            scanner::scan_folder(db, job_signal_tx, event_tx, &folder_id).await?;
        }
        "ingest_pdf" => {
            let document_id = payload
                .document_id
                .ok_or_else(|| anyhow::anyhow!("ingest_pdf missing document_id"))?;
            pipeline::ingest_pdf(db, asset_store, tantivy_tx, event_tx, &document_id).await?;
        }
        "reindex_document" => {
            let document_id = payload
                .document_id
                .ok_or_else(|| anyhow::anyhow!("reindex_document missing document_id"))?;
            tantivy_tx
                .send(IndexCommand::UpsertDocument { document_id })
                .await?;
        }
        "render_cover" => {}
        other => anyhow::bail!("unknown job kind: {other}"),
    }
    Ok(())
}

fn is_permanent_job_error(message: &str) -> bool {
    message.contains("not a valid PDF")
        || message.contains("too small to be a valid PDF")
        || message.contains("invalid PDF header")
        || message.contains("unsupported document extension")
        || message.contains("couldn't parse input: invalid file header")
}
