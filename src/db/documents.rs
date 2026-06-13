use std::path::Path;

use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    db::models::{Document, DocumentStatus},
    util::time::now_rfc3339,
};

#[derive(Debug, Clone)]
pub struct UpsertDocument {
    pub folder_id: String,
    pub path: String,
    pub canonical_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub modified_at: String,
}

pub async fn upsert_discovered(db: &SqlitePool, input: UpsertDocument) -> anyhow::Result<String> {
    let existing = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE path = ?")
        .bind(&input.path)
        .fetch_optional(db)
        .await?;
    let now = now_rfc3339();
    if let Some(document) = existing {
        sqlx::query(
            r#"
            UPDATE documents
            SET folder_id = ?, canonical_path = ?, file_name = ?, file_size = ?,
                modified_at = ?, status = CASE WHEN status = 'ready' THEN status ELSE 'discovered' END,
                error = NULL, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&input.folder_id)
        .bind(&input.canonical_path)
        .bind(&input.file_name)
        .bind(input.file_size)
        .bind(&input.modified_at)
        .bind(&now)
        .bind(&document.id)
        .execute(db)
        .await?;
        return Ok(document.id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO documents
          (id, folder_id, path, canonical_path, file_name, file_size, modified_at, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&input.folder_id)
    .bind(&input.path)
    .bind(&input.canonical_path)
    .bind(&input.file_name)
    .bind(input.file_size)
    .bind(&input.modified_at)
    .bind(DocumentStatus::Discovered.as_str())
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(id)
}

pub async fn get(db: &SqlitePool, id: &str) -> anyhow::Result<Document> {
    Ok(
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await?,
    )
}

pub async fn find_by_path(db: &SqlitePool, path: &Path) -> anyhow::Result<Option<Document>> {
    Ok(
        sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE path = ?")
            .bind(path.to_string_lossy().to_string())
            .fetch_optional(db)
            .await?,
    )
}

pub async fn mark_missing_by_path(db: &SqlitePool, path: &Path) -> anyhow::Result<Option<String>> {
    let Some(document) = find_by_path(db, path).await? else {
        return Ok(None);
    };
    sqlx::query("UPDATE documents SET status = 'missing', updated_at = ? WHERE id = ?")
        .bind(now_rfc3339())
        .bind(&document.id)
        .execute(db)
        .await?;
    Ok(Some(document.id))
}

pub async fn mark_folder_missing_except(
    db: &SqlitePool,
    folder_id: &str,
    seen_paths: &[String],
) -> anyhow::Result<()> {
    let docs = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE folder_id = ?")
        .bind(folder_id)
        .fetch_all(db)
        .await?;
    let seen: std::collections::HashSet<&str> = seen_paths.iter().map(String::as_str).collect();
    let now = now_rfc3339();
    for doc in docs {
        if !seen.contains(doc.path.as_str()) {
            sqlx::query("UPDATE documents SET status = 'missing', updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(doc.id)
                .execute(db)
                .await?;
        }
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct DocumentCard {
    pub id: String,
    pub path: String,
    pub title: Option<String>,
    pub file_name: String,
    pub status: String,
    pub page_count: Option<i64>,
    pub classification: Option<serde_json::Value>,
    pub cover_url: String,
}
