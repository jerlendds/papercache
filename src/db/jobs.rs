use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::{Row, SqlitePool};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    app_state::JobSignal,
    db::models::{Job, JobId, JobKind, JobStatus},
    util::time::now_rfc3339,
    web::events::AppEvent,
};

pub async fn enqueue_job(
    db: &SqlitePool,
    tx: &mpsc::Sender<JobSignal>,
    kind: JobKind,
    payload: Value,
    priority: i64,
) -> anyhow::Result<JobId> {
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO jobs (id, kind, payload_json, status, priority, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(kind.as_str())
    .bind(payload.to_string())
    .bind(JobStatus::Queued.as_str())
    .bind(priority)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    let _ = tx.send(JobSignal::NewWork).await;
    Ok(id)
}

pub async fn claim_next_job(db: &SqlitePool) -> anyhow::Result<Option<Job>> {
    let now = now_rfc3339();
    let mut tx = db.begin().await?;
    let row = sqlx::query(
        r#"
        SELECT id FROM jobs
        WHERE status = 'queued'
          AND (run_after IS NULL OR run_after <= ?)
        ORDER BY priority DESC, created_at ASC
        LIMIT 1
        "#,
    )
    .bind(&now)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(None);
    };
    let id: String = row.try_get("id")?;

    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'running', locked_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&id)
    .execute(&mut *tx)
    .await?;

    let job = sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = ?")
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some(job))
}

pub async fn reset_stale_running_jobs(db: &SqlitePool) -> anyhow::Result<()> {
    let now = now_rfc3339();
    sqlx::query(
        r#"
        UPDATE jobs
        SET status = 'queued', locked_at = NULL, updated_at = ?
        WHERE status = 'running'
        "#,
    )
    .bind(now)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_started(
    db: &SqlitePool,
    event_tx: &tokio::sync::broadcast::Sender<AppEvent>,
    job: &Job,
) -> anyhow::Result<()> {
    create_event(db, &job.id, "job_started", None, None).await?;
    let _ = event_tx.send(AppEvent::JobStarted {
        job_id: job.id.clone(),
        kind: job.kind.clone(),
    });
    Ok(())
}

pub async fn mark_succeeded(db: &SqlitePool, job: &Job) -> anyhow::Result<()> {
    let now = now_rfc3339();
    sqlx::query(
        "UPDATE jobs SET status = 'succeeded', error = NULL, locked_at = NULL, updated_at = ? WHERE id = ?",
    )
    .bind(&now)
    .bind(&job.id)
    .execute(db)
    .await?;
    create_event(db, &job.id, "job_succeeded", None, None).await?;
    Ok(())
}

pub async fn mark_failed_or_retry(
    db: &SqlitePool,
    tx: &mpsc::Sender<JobSignal>,
    event_tx: &tokio::sync::broadcast::Sender<AppEvent>,
    job: &Job,
    error: &str,
    permanent: bool,
) -> anyhow::Result<()> {
    let attempts = job.attempts + 1;
    let now = Utc::now();
    let should_retry = !permanent && attempts < job.max_attempts;
    let status = if should_retry {
        JobStatus::Queued
    } else {
        JobStatus::Failed
    };
    let run_after = if should_retry {
        Some((now + backoff(attempts)).to_rfc3339())
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE jobs
        SET status = ?, attempts = ?, error = ?, run_after = ?, locked_at = NULL, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status.as_str())
    .bind(attempts)
    .bind(error)
    .bind(run_after)
    .bind(now.to_rfc3339())
    .bind(&job.id)
    .execute(db)
    .await?;

    create_event(db, &job.id, "job_failed", Some(error), None).await?;
    let _ = event_tx.send(AppEvent::JobFailed {
        job_id: job.id.clone(),
        error: error.to_string(),
    });
    if should_retry {
        let tx = tx.clone();
        let delay = backoff(attempts).to_std().unwrap_or_default();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = tx.send(JobSignal::NewWork).await;
        });
    }
    Ok(())
}

fn backoff(attempts: i64) -> Duration {
    match attempts {
        0 | 1 => Duration::seconds(5),
        2 => Duration::seconds(30),
        _ => Duration::minutes(5),
    }
}

pub async fn create_event(
    db: &SqlitePool,
    job_id: &str,
    event_type: &str,
    message: Option<&str>,
    metadata: Option<Value>,
) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO job_events (id, job_id, event_type, message, metadata_json, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(job_id)
    .bind(event_type)
    .bind(message)
    .bind(metadata.map(|value| value.to_string()))
    .bind(now_rfc3339())
    .execute(db)
    .await?;
    Ok(())
}
