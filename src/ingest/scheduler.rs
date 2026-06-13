use sqlx::SqlitePool;
use tokio::sync::mpsc;

use crate::{
    app_state::{JobPayload, JobSignal},
    db::{folders, jobs::enqueue_job, models::JobKind},
};

pub async fn run_periodic_scans(db: SqlitePool, job_signal_tx: mpsc::Sender<JobSignal>) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30 * 60));
    loop {
        interval.tick().await;
        match folders::list_enabled(&db).await {
            Ok(folders) => {
                for folder in folders {
                    if let Err(error) = enqueue_job(
                        &db,
                        &job_signal_tx,
                        JobKind::ScanFolder,
                        JobPayload::scan_folder(folder.id),
                        -10,
                    )
                    .await
                    {
                        tracing::warn!(?error, "failed to enqueue periodic scan");
                    }
                }
            }
            Err(error) => tracing::warn!(?error, "failed to list folders for periodic scan"),
        }
    }
}
