use std::path::Path;

use serde::Serialize;
use sqlx::{QueryBuilder, Row, Sqlite, SqlitePool};
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

pub async fn upsert_discovered_batch(
    db: &SqlitePool,
    inputs: &[UpsertDocument],
) -> anyhow::Result<Vec<String>> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }

    let now = now_rfc3339();
    let rows = inputs
        .iter()
        .map(|input| (Uuid::new_v4().to_string(), input))
        .collect::<Vec<_>>();
    let mut builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r#"
        INSERT INTO documents
          (id, folder_id, path, canonical_path, file_name, file_size, modified_at, status, created_at, updated_at)
        "#,
    );

    builder.push_values(rows.iter(), |mut row, (id, input)| {
        row.push_bind(id)
            .push_bind(&input.folder_id)
            .push_bind(&input.path)
            .push_bind(&input.canonical_path)
            .push_bind(&input.file_name)
            .push_bind(input.file_size)
            .push_bind(&input.modified_at)
            .push_bind(DocumentStatus::Discovered.as_str())
            .push_bind(&now)
            .push_bind(&now);
    });

    builder.push(
        r#"
        ON CONFLICT(path) DO UPDATE SET
          folder_id = excluded.folder_id,
          canonical_path = excluded.canonical_path,
          file_name = excluded.file_name,
          file_size = excluded.file_size,
          modified_at = excluded.modified_at,
          status = CASE
            WHEN documents.status = 'ready' THEN documents.status
            WHEN documents.status = 'failed'
              AND documents.file_size = excluded.file_size
              AND documents.modified_at = excluded.modified_at
            THEN documents.status
            ELSE excluded.status
          END,
          error = CASE
            WHEN documents.status = 'failed'
              AND documents.file_size = excluded.file_size
              AND documents.modified_at = excluded.modified_at
            THEN documents.error
            ELSE NULL
          END,
          updated_at = excluded.updated_at
        RETURNING id
        "#,
    );

    let rows = builder.build().fetch_all(db).await?;
    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("id"))
        .collect())
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

pub async fn mark_failed(db: &SqlitePool, id: &str, error: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE documents SET status = 'failed', error = ?, updated_at = ? WHERE id = ?")
        .bind(error)
        .bind(now_rfc3339())
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
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
