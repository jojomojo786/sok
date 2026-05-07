use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::views::IndexTemplate;

async fn index(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let _client = pool.get().await?;
    let html = IndexTemplate.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("healthy")
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/health", web::get().to(health_check));
}
