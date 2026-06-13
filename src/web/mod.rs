pub mod assets;
pub mod documents;
pub mod events;
pub mod folders;
pub mod health;
pub mod jobs;
pub mod search;

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
        );
}

pub fn require_write_auth(req: &HttpRequest, state: &AppState) -> Result<(), AppError> {
    let header_ok = req
        .headers()
        .get("X-Local-Auth")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == state.auth_token);
    let cookie_ok = req
        .cookie("papercache_auth")
        .is_some_and(|cookie| cookie.value() == state.auth_token);
    if header_ok || cookie_ok {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
