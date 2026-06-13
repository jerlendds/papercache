pub mod assets;
pub mod chunks;
pub mod documents;
pub mod folders;
pub mod jobs;
pub mod models;

use std::{str::FromStr, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use uuid::Uuid;

pub async fn open_sqlite(database_url: &str) -> anyhow::Result<SqlitePool> {
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if !std::path::Path::new(path).exists() {
            std::fs::File::create(path)?;
        }
    }

    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    Ok(pool)
}

pub async fn get_or_create_auth_token(db: &SqlitePool) -> anyhow::Result<String> {
    if let Some(value) = sqlx::query_scalar::<_, String>(
        "SELECT value_json FROM app_settings WHERE key = 'local_auth_token'",
    )
    .fetch_optional(db)
    .await?
    {
        return Ok(serde_json::from_str(&value)?);
    }

    let token = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO app_settings (key, value_json, updated_at)
        VALUES ('local_auth_token', ?, ?)
        "#,
    )
    .bind(serde_json::to_string(&token)?)
    .bind(crate::util::time::now_rfc3339())
    .execute(db)
    .await?;
    Ok(token)
}
