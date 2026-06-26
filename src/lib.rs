pub mod config;
pub mod db;
pub mod errors;
pub mod fixtures;
pub mod handlers;
pub mod logging;
pub mod models;
pub mod views;

use actix_files as fs;
use actix_web::dev::ServiceRequest;
use actix_web::{middleware::Compress, web, App, HttpRequest};
use tracing_actix_web::TracingLogger;

use crate::config::Config;
use crate::db::DbPool;

/// Production-compatible static mounts used by the server and integration tests.
///
/// - `/static` — entire `static/` tree (`/static/js`, `/static/fox-tpl`, manifest, etc.)
/// - `/fox-tpl` — mirror of `static/fox-tpl` for categories-style template paths
/// - `/site.webmanifest` — root manifest URL used by mirrored HTML (file lives under `static/`)
pub fn configure_static(cfg: &mut web::ServiceConfig) {
    cfg.service(fs::Files::new("/static", "static").show_files_listing())
        .service(fs::Files::new("/fox-tpl", "static/fox-tpl"))
        .service(fs::Files::new("/static/fox-tpl", "static/fox-tpl"))
        .route("/site.webmanifest", web::get().to(serve_site_webmanifest))
        .route(
            "/favicon.ico",
            web::get().to(|| serve_static_root_file("favicon.ico")),
        )
        .route(
            "/apple-touch-icon.png",
            web::get().to(|| serve_static_root_file("apple-touch-icon.png")),
        )
        .route(
            "/favicon-32x32.png",
            web::get().to(|| serve_static_root_file("favicon-32x32.png")),
        )
        .route(
            "/favicon-16x16.png",
            web::get().to(|| serve_static_root_file("favicon-16x16.png")),
        )
        .route(
            "/safari-pinned-tab.svg",
            web::get().to(|| serve_static_root_file("safari-pinned-tab.svg")),
        )
        .route(
            "/android-chrome-192x192.png",
            web::get().to(|| serve_static_root_file("android-chrome-192x192.png")),
        )
        .route(
            "/android-chrome-512x512.png",
            web::get().to(|| serve_static_root_file("android-chrome-512x512.png")),
        );
}

async fn serve_site_webmanifest(_req: HttpRequest) -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/site.webmanifest")?)
}

async fn serve_static_root_file(file_name: &'static str) -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open(format!("static/{file_name}"))?)
}

/// Builds the Actix `App` with static mounts and routes (used by integration tests and optional reuse).
pub fn build_app(
    cfg: Config,
    pool: DbPool,
) -> App<impl actix_web::dev::ServiceFactory<ServiceRequest>> {
    App::new()
        .wrap(TracingLogger::default())
        .wrap(Compress::default())
        .app_data(web::Data::new(pool))
        .app_data(web::Data::new(cfg))
        .configure(configure_static)
        .configure(handlers::routes)
}
