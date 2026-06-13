use actix_web::{HttpRequest, HttpResponse, Scope, web};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    app_state::{AppState, JobPayload},
    db::{folders, jobs::enqueue_job, models::JobKind},
    error::{AppError, AppResult},
    web::require_write_auth,
};

pub fn routes() -> Scope {
    web::scope("/folders")
        .route("", web::post().to(post))
        .route("", web::get().to(list))
        .route("/{id}", web::delete().to(delete))
}

#[derive(Debug, Deserialize)]
struct ImportFolderRequest {
    path: String,
    recursive: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ImportFolderResponse {
    folder_id: String,
    status: &'static str,
}

async fn post(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<ImportFolderRequest>,
) -> AppResult<HttpResponse> {
    require_write_auth(&req, &state)?;
    let path = std::path::PathBuf::from(&body.path);
    if !path.exists() || !path.is_dir() {
        return Err(AppError::BadRequest(
            "path must be an existing directory".to_string(),
        ));
    }
    let canonical = path.canonicalize()?;
    let canonical_str = canonical.to_string_lossy().to_string();
    let folder_id =
        folders::upsert_folder(&state.db, &canonical_str, body.recursive.unwrap_or(true)).await?;
    let job_id = enqueue_job(
        &state.db,
        &state.job_signal_tx,
        JobKind::ScanFolder,
        JobPayload::scan_folder(folder_id.clone()),
        10,
    )
    .await?;
    let _ = state
        .event_tx
        .send(crate::web::events::AppEvent::JobQueued {
            job_id,
            kind: JobKind::ScanFolder.as_str().to_string(),
        });
    Ok(HttpResponse::Accepted().json(ImportFolderResponse {
        folder_id,
        status: "queued",
    }))
}

#[derive(Debug, Serialize)]
struct FolderResponse {
    id: String,
    path: String,
    recursive: bool,
    enabled: bool,
    last_scan_at: Option<String>,
    document_count: i64,
}

async fn list(state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT f.id, f.path, f.recursive, f.enabled, f.last_scan_at, COUNT(d.id) AS document_count
        FROM imported_folders f
        LEFT JOIN documents d ON d.folder_id = f.id
        GROUP BY f.id
        ORDER BY f.created_at ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    let folders = rows
        .into_iter()
        .map(|row| FolderResponse {
            id: row.get("id"),
            path: row.get("path"),
            recursive: row.get::<i64, _>("recursive") == 1,
            enabled: row.get::<i64, _>("enabled") == 1,
            last_scan_at: row.get("last_scan_at"),
            document_count: row.get("document_count"),
        })
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(folders))
}

async fn delete(
    req: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    require_write_auth(&req, &state)?;
    folders::disable_folder(&state.db, &id).await?;
    Ok(HttpResponse::Accepted().json(serde_json::json!({ "status": "disabled" })))
}
