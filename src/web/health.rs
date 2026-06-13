use actix_web::{HttpResponse, Responder};

pub async fn get() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "ok": true }))
}
