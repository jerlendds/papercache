use std::path::PathBuf;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::util::time::now_rfc3339;

#[derive(Debug)]
pub struct AssetStore {
    covers_dir: PathBuf,
}

impl AssetStore {
    pub fn new(covers_dir: PathBuf) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&covers_dir)?;
        Ok(Self { covers_dir })
    }

    pub fn cover_path(&self, document_id: &str, hash: &str) -> PathBuf {
        let prefix = &hash[..hash.len().min(12)];
        self.covers_dir.join(format!("{document_id}-{prefix}.webp"))
    }

    pub fn relative_cover_path(document_id: &str, hash: &str) -> String {
        let prefix = &hash[..hash.len().min(12)];
        format!("covers/{document_id}-{prefix}.webp")
    }

    pub fn resolve_relative(&self, relative: &str) -> Option<PathBuf> {
        let name = relative.strip_prefix("covers/")?;
        if name.contains('/') || name.contains('\\') || name.contains("..") {
            return None;
        }
        Some(self.covers_dir.join(name))
    }
}

pub async fn upsert_cover_asset(
    db: &SqlitePool,
    document_id: &str,
    relative_path: &str,
    sha256: Option<&str>,
) -> anyhow::Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = now_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO assets (id, document_id, kind, mime, path, sha256, created_at)
        VALUES (?, ?, 'cover', 'image/webp', ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(document_id)
    .bind(relative_path)
    .bind(sha256)
    .bind(&now)
    .execute(db)
    .await?;

    sqlx::query("UPDATE documents SET cover_asset_id = ?, updated_at = ? WHERE id = ?")
        .bind(&id)
        .bind(&now)
        .bind(document_id)
        .execute(db)
        .await?;
    Ok(id)
}
