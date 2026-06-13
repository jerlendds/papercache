pub mod assets;
pub mod documents;
pub mod events;
pub mod folders;
pub mod health;
pub mod jobs;
pub mod search;
pub mod ui;

use actix_web::{HttpRequest, web};

use crate::{app_state::AppState, error::AppError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/api/health").route(web::get().to(health::get)))
        .service(
            web::scope("/api")
                .service(folders::routes())
                .service(documents::routes())
                .service(jobs::routes())
                .service(search::routes())
                .service(events::routes()),
        )
        .default_service(web::route().to(ui::serve));
}

pub fn require_write_auth(req: &HttpRequest, state: &AppState) -> Result<(), AppError> {
    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim);
    if token.is_some_and(|value| value == state.auth_token) {
        Ok(())
    } else {
        tracing::warn!(
            path = req.path(),
            has_authorization_header = req
                .headers()
                .contains_key(actix_web::http::header::AUTHORIZATION),
            "write request rejected by local auth"
        );
        Err(AppError::Unauthorized)
    }
}
