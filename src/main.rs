use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use actix_web::{middleware::Compress, web, App, HttpServer};
use sok::{config, configure_static, db, handlers, logging};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    logging::init();

    let cfg = match config::Config::load() {
        Ok(cfg) => cfg,
        Err(err) => {
            tracing::error!(error = %err, "configuration load failed");
            std::process::exit(1);
        }
    };
    tracing::info!(bind = %cfg.bind_addr, db = %cfg.database_log_label(), "starting sok server");
    let pool = db::create_pool(&cfg).await;

    let bind_addr = cfg.bind_addr.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(Compress::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cfg.clone()))
            .configure(configure_static)
            .configure(handlers::routes)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
