use actix_web::{HttpRequest, HttpResponse, Scope, http::header, web};
use serde::Deserialize;
use sqlx::{QueryBuilder, Row, Sqlite};

use crate::{
    app_state::AppState,
    db::{documents::DocumentCard, models::Document},
    error::{AppError, AppResult},
    web::require_write_auth,
};

pub fn routes() -> Scope {
    web::scope("/documents")
        .route("", web::get().to(list))
        .route("/{id}", web::get().to(get))
        .route("/{id}/classification", web::put().to(put_classification))
        .route(
            "/{id}/classification",
            web::delete().to(delete_classification),
        )
        .route("/{id}/cover", web::get().to(cover))
        .route("/{id}/file", web::get().to(file))
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    status: Option<String>,
    folder_id: Option<String>,
    q: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list(state: web::Data<AppState>, query: web::Query<ListQuery>) -> AppResult<HttpResponse> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let mut builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT * FROM documents WHERE 1 = 1");
    if let Some(status) = &query.status {
        builder.push(" AND status = ").push_bind(status);
    }
    if let Some(folder_id) = &query.folder_id {
        builder.push(" AND folder_id = ").push_bind(folder_id);
    }
    if let Some(q) = &query.q {
        let pattern = format!("%{q}%");
        builder
            .push(" AND (title LIKE ")
            .push_bind(pattern.clone())
            .push(" OR file_name LIKE ")
            .push_bind(pattern.clone())
            .push(" OR path LIKE ")
            .push_bind(pattern)
            .push(" OR doi LIKE ")
            .push_bind(format!("%{q}%"))
            .push(" OR arxiv_id LIKE ")
            .push_bind(format!("%{q}%"))
            .push(" OR authors_json LIKE ")
            .push_bind(format!("%{q}%"))
            .push(")");
    }
    builder
        .push(" ORDER BY updated_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let docs = builder
        .build_query_as::<Document>()
        .fetch_all(&state.db)
        .await?;
    let cards = docs.into_iter().map(document_card).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(cards))
}

async fn get(state: web::Data<AppState>, id: web::Path<String>) -> AppResult<HttpResponse> {
    let doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
        .bind(id.as_str())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": doc.id,
        "folder_id": doc.folder_id,
        "path": doc.path,
        "canonical_path": doc.canonical_path,
        "file_name": doc.file_name,
        "file_size": doc.file_size,
        "modified_at": doc.modified_at,
        "sha256": doc.sha256,
        "title": doc.title,
        "authors": parse_authors(doc.authors_json.as_deref()),
        "year": doc.year,
        "doi": doc.doi,
        "arxiv_id": doc.arxiv_id,
        "page_count": doc.page_count,
        "status": doc.status,
        "error": doc.error,
        "classification": doc.classification_json.as_deref().and_then(|v| serde_json::from_str::<serde_json::Value>(v).ok()).unwrap_or_else(|| serde_json::json!(null)),
        "cover_url": format!("/api/documents/{}/cover", doc.id),
        "file_url": format!("/api/documents/{}/file", doc.id),
        "created_at": doc.created_at,
        "updated_at": doc.updated_at,
    })))
}

async fn put_classification(
    req: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> AppResult<HttpResponse> {
    require_write_auth(&req, &state)?;
    sqlx::query("UPDATE documents SET classification_json = ?, updated_at = ? WHERE id = ?")
        .bind(body.to_string())
        .bind(crate::util::time::now_rfc3339())
        .bind(id.as_str())
        .execute(&state.db)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "updated" })))
}

async fn delete_classification(
    req: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> AppResult<HttpResponse> {
    require_write_auth(&req, &state)?;
    sqlx::query("UPDATE documents SET classification_json = NULL, updated_at = ? WHERE id = ?")
        .bind(crate::util::time::now_rfc3339())
        .bind(id.as_str())
        .execute(&state.db)
        .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "removed" })))
}

async fn cover(state: web::Data<AppState>, id: web::Path<String>) -> AppResult<HttpResponse> {
    let row = sqlx::query(
        r#"
        SELECT a.path, a.mime
        FROM assets a
        JOIN documents d ON d.cover_asset_id = a.id
        WHERE d.id = ? AND a.kind = 'cover'
        "#,
    )
    .bind(id.as_str())
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;
    let relative: String = row.get("path");
    let mime: String = row.get("mime");
    let path = state
        .asset_store
        .resolve_relative(&relative)
        .ok_or(AppError::NotFound)?;
    let bytes = tokio::fs::read(path).await?;
    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, mime))
        .body(bytes))
}

async fn file(state: web::Data<AppState>, id: web::Path<String>) -> AppResult<HttpResponse> {
    let doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
        .bind(id.as_str())
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let path = std::path::PathBuf::from(&doc.path);
    if !path.exists() || !path.is_file() {
        return Err(AppError::NotFound);
    }
    let bytes = tokio::fs::read(path).await?;
    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/pdf"))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", doc.file_name.replace('"', "")),
        ))
        .body(bytes))
}

fn document_card(doc: Document) -> DocumentCard {
    let id = doc.id;
    DocumentCard {
        cover_url: format!("/api/documents/{id}/cover"),
        file_url: format!("/api/documents/{id}/file"),
        classification: doc
            .classification_json
            .as_deref()
            .and_then(|value| serde_json::from_str(value).ok()),
        authors: parse_authors(doc.authors_json.as_deref()),
        id,
        folder_id: doc.folder_id,
        path: doc.path,
        canonical_path: doc.canonical_path,
        title: doc.title,
        file_name: doc.file_name,
        file_size: doc.file_size,
        modified_at: doc.modified_at,
        sha256: doc.sha256,
        year: doc.year,
        doi: doc.doi,
        arxiv_id: doc.arxiv_id,
        status: doc.status,
        error: doc.error,
        page_count: doc.page_count,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    }
}

fn parse_authors(value: Option<&str>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };

    match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Array(items)) => items
            .into_iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect(),
        Ok(serde_json::Value::String(author)) => vec![author],
        _ => Vec::new(),
    }
}
