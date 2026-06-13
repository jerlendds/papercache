use actix_web::{HttpResponse, Scope, web};

use crate::{app_state::AppState, error::AppResult};

pub fn routes() -> Scope {
    web::scope("/jobs").route("", web::get().to(list))
}

async fn list(state: web::Data<AppState>) -> AppResult<HttpResponse> {
    let rows = sqlx::query_as::<_, crate::db::models::Job>(
        "SELECT * FROM jobs ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(
        rows.into_iter()
            .map(|job| {
                serde_json::json!({
                    "id": job.id,
                    "kind": job.kind,
                    "status": job.status,
                    "priority": job.priority,
                    "attempts": job.attempts,
                    "max_attempts": job.max_attempts,
                    "error": job.error,
                    "run_after": job.run_after,
                    "locked_at": job.locked_at,
                    "created_at": job.created_at,
                    "updated_at": job.updated_at,
                })
            })
            .collect::<Vec<_>>(),
    ))
}
