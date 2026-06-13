use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::{db::models::ImportedFolder, util::time::now_rfc3339};

pub async fn upsert_folder(db: &SqlitePool, path: &str, recursive: bool) -> anyhow::Result<String> {
    let existing = sqlx::query("SELECT id FROM imported_folders WHERE path = ?")
        .bind(path)
        .fetch_optional(db)
        .await?;
    let now = now_rfc3339();
    if let Some(row) = existing {
        let id: String = row.try_get("id")?;
        sqlx::query(
            "UPDATE imported_folders SET recursive = ?, enabled = 1, updated_at = ? WHERE id = ?",
        )
        .bind(recursive as i64)
        .bind(&now)
        .bind(&id)
        .execute(db)
        .await?;
        return Ok(id);
    }

    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO imported_folders (id, path, recursive, enabled, created_at, updated_at)
        VALUES (?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(path)
    .bind(recursive as i64)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;
    Ok(id)
}

pub async fn get_folder(db: &SqlitePool, id: &str) -> anyhow::Result<ImportedFolder> {
    Ok(
        sqlx::query_as::<_, ImportedFolder>("SELECT * FROM imported_folders WHERE id = ?")
            .bind(id)
            .fetch_one(db)
            .await?,
    )
}

pub async fn list_enabled(db: &SqlitePool) -> anyhow::Result<Vec<ImportedFolder>> {
    Ok(
        sqlx::query_as::<_, ImportedFolder>("SELECT * FROM imported_folders WHERE enabled = 1")
            .fetch_all(db)
            .await?,
    )
}

pub async fn touch_scan(db: &SqlitePool, id: &str) -> anyhow::Result<()> {
    let now = now_rfc3339();
    sqlx::query("UPDATE imported_folders SET last_scan_at = ?, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn disable_folder(db: &SqlitePool, id: &str) -> anyhow::Result<()> {
    let now = now_rfc3339();
    sqlx::query("UPDATE imported_folders SET enabled = 0, updated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(db)
        .await?;
    sqlx::query("UPDATE documents SET status = 'missing', updated_at = ? WHERE folder_id = ?")
        .bind(&now)
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}
