use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::views::{CategoriesTemplate, IndexTemplate};

async fn index(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    sqlx::query("SELECT 1").execute(pool.get_ref()).await?;
    let html = IndexTemplate.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

async fn categories(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    sqlx::query("SELECT 1").execute(pool.get_ref()).await?;
    let html = CategoriesTemplate.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("healthy")
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(index))
        .route("/categories", web::get().to(categories))
        .route("/health", web::get().to(health_check));
}
