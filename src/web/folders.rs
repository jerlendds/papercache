use std::path::PathBuf;

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
    tracing::info!(
        path = %body.path,
        recursive = body.recursive.unwrap_or(true),
        "folder import requested"
    );
    let path = normalize_import_path(&body.path);
    if !path.exists() || !path.is_dir() {
        tracing::warn!(
            path = %body.path,
            resolved_path = %path.display(),
            exists = path.exists(),
            is_dir = path.is_dir(),
            "folder import rejected"
        );
        return Err(AppError::BadRequest(format!(
            "path must be an existing directory: {}",
            path.display()
        )));
    }
    let canonical = path.canonicalize().map_err(|error| {
        tracing::warn!(
            path = %body.path,
            ?error,
            "folder import canonicalization failed"
        );
        error
    })?;
    let canonical_str = canonical.to_string_lossy().to_string();
    tracing::info!(
        path = %body.path,
        canonical_path = %canonical_str,
        "folder import path canonicalized"
    );
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
    tracing::info!(
        folder_id = %folder_id,
        kind = JobKind::ScanFolder.as_str(),
        "folder import scan queued"
    );
    Ok(HttpResponse::Accepted().json(ImportFolderResponse {
        folder_id,
        status: "queued",
    }))
}

fn normalize_import_path(value: &str) -> PathBuf {
    let trimmed = value.trim();
    let unquoted = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(trimmed);

    if unquoted == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    }
    if let Some(rest) = unquoted
        .strip_prefix("~/")
        .or_else(|| unquoted.strip_prefix("~\\"))
    {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }

    PathBuf::from(unquoted)
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
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

#[cfg(test)]
mod tests {
    use super::normalize_import_path;

    #[test]
    fn normalize_import_path_strips_wrapping_quotes() {
        assert_eq!(
            normalize_import_path("\"/tmp/papers\"").to_string_lossy(),
            "/tmp/papers"
        );
        assert_eq!(
            normalize_import_path("'/tmp/papers'").to_string_lossy(),
            "/tmp/papers"
        );
    }
}
