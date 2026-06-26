use actix_web::{HttpResponse, Responder};

use super::common::HANDLER_MARKER;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "health"))
        .body("healthy")
}
