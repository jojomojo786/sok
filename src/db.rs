use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

use crate::config::Config;

pub type DbPool = MySqlPool;

pub async fn create_pool(cfg: &Config) -> DbPool {
    MySqlPoolOptions::new()
        .max_connections(16)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to create database pool")
}
