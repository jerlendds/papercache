use actix_web::{HttpResponse, Scope, web};
use serde::Serialize;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppEvent {
    JobQueued {
        job_id: String,
        kind: String,
    },
    JobStarted {
        job_id: String,
        kind: String,
    },
    FolderScanCompleted {
        folder_id: String,
        discovered: usize,
    },
    DocumentDiscovered {
        document_id: String,
        folder_id: String,
    },
    DocumentStage {
        document_id: String,
        stage: String,
    },
    DocumentReady {
        document_id: String,
        folder_id: Option<String>,
    },
    JobFailed {
        job_id: String,
        error: String,
    },
}

pub fn routes() -> Scope {
    web::scope("/events").route("", web::get().to(get))
}

async fn get(state: web::Data<AppState>) -> HttpResponse {
    let mut rx = state.event_tx.subscribe();
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());
                    yield Ok::<_, actix_web::Error>(web::Bytes::from(format!("data: {json}\n\n")));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .streaming(stream)
}
