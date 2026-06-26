use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use crate::config::Config;
use crate::logging::{log_db_pool_connecting, log_db_pool_failed, log_db_pool_ready};

pub type DbPool = MySqlPool;

pub async fn create_pool(cfg: &Config) -> DbPool {
    let db_label = cfg.database_log_label();
    log_db_pool_connecting(&db_label);

    match MySqlPoolOptions::new()
        .max_connections(16)
        .connect(&cfg.database_url)
        .await
    {
        Ok(pool) => {
            log_db_pool_ready(&db_label);
            pool
        }
        Err(e) => {
            log_db_pool_failed(&db_label, &e);
            panic!("Failed to create database pool");
        }
    }
}
