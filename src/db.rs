use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;

use crate::config::Config;

pub type DbPool = Pool;

pub fn create_pool(cfg: &Config) -> DbPool {
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Verified,
    };
    let mgr = Manager::from_config(cfg.db_dsn().parse().unwrap(), NoTls, mgr_config);
    Pool::builder(mgr)
        .max_size(16)
        .build()
        .expect("Failed to create database pool")
}
