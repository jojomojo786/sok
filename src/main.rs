use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod config;
mod db;
mod errors;
mod handlers;
mod models;
mod views;

use actix_web::{web, App, HttpServer, middleware::Compress};
use actix_files as fs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let cfg = config::Config::load();
    let pool = db::create_pool(&cfg);

    let bind_addr = cfg.bind_addr.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(Compress::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .service(fs::Files::new("/static", "static").show_files_listing())
            .service(fs::Files::new("/fox-tpl", "static/fox-tpl"))
            .configure(handlers::routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
