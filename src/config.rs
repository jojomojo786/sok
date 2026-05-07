use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigFile {
    db: DbConfig,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub db: DbConfig,
    pub bind_addr: String,
}

impl Config {
    pub fn load() -> Self {
        let file = std::fs::read_to_string("config.json").expect("Failed to read config.json");
        let cf: ConfigFile = serde_json::from_str(&file).expect("Failed to parse config.json");
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        Config { db: cf.db, bind_addr }
    }

    pub fn db_dsn(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.db.host, self.db.port, self.db.user, self.db.password, self.db.database
        )
    }
}
