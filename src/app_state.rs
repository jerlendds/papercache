use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};

use crate::{index::commands::IndexCommand, web::events::AppEvent};

#[derive(Debug, Clone)]
pub enum JobSignal {
    NewWork,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub job_signal_tx: mpsc::Sender<JobSignal>,
    pub event_tx: broadcast::Sender<AppEvent>,
    pub asset_store: Arc<crate::db::assets::AssetStore>,
    #[allow(dead_code)]
    pub tantivy_tx: mpsc::Sender<IndexCommand>,
    pub auth_token: String,
    pub tantivy_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JobPayload {
    pub folder_id: Option<String>,
    pub document_id: Option<String>,
}

impl JobPayload {
    pub fn scan_folder(folder_id: String) -> serde_json::Value {
        serde_json::json!({ "folder_id": folder_id })
    }

    pub fn ingest_pdf(document_id: String) -> serde_json::Value {
        serde_json::json!({ "document_id": document_id })
    }
}
