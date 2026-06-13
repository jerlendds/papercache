use actix_web::{HttpResponse, Scope, web};
use serde::Deserialize;

use crate::{app_state::AppState, error::AppResult, index::tantivy_index};

pub fn routes() -> Scope {
    web::scope("/search").route("", web::post().to(post))
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
}

async fn post(
    state: web::Data<AppState>,
    body: web::Json<SearchRequest>,
) -> AppResult<HttpResponse> {
    let index_dir = state.tantivy_dir.clone();
    let query = body.query.clone();
    let limit = body.limit.unwrap_or(20).min(100);
    let offset = body.offset.unwrap_or(0);
    let results = tokio::task::spawn_blocking(move || {
        tantivy_index::search(index_dir, &query, limit, offset)
    })
    .await
    .map_err(anyhow::Error::from)??;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "results": results })))
}
