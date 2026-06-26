use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
}

impl Config {
    pub fn load() -> Self {
        let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let database_url = std::env::var("DATABASE_URL")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(Self::database_url_from_config_json)
            .expect("Set DATABASE_URL (mysql://...) in .env or db fields in config.json");

        Config {
            database_url: Self::normalize_mysql_url(&database_url),
            bind_addr,
        }
    }

    fn database_url_from_config_json() -> Option<String> {
        #[derive(Deserialize)]
        struct DbConfig {
            host: String,
            port: u16,
            database: String,
            user: String,
            password: String,
        }
        #[derive(Deserialize)]
        struct ConfigFile {
            db: DbConfig,
        }

        let file = std::fs::read_to_string("config.json").ok()?;
        let cf: ConfigFile = serde_json::from_str(&file).ok()?;
        if cf.db.password.is_empty() {
            return None;
        }

        Some(format!(
            "mysql://{}:{}@{}:{}/{}",
            urlencoding::encode(&cf.db.user),
            urlencoding::encode(&cf.db.password),
            cf.db.host,
            cf.db.port,
            cf.db.database
        ))
    }

    fn normalize_mysql_url(raw: &str) -> String {
        let mut parsed = Url::parse(raw).expect("DATABASE_URL must be a valid URL");
        if parsed.scheme() != "mysql" {
            panic!("DATABASE_URL must use mysql:// scheme");
        }
        parsed.set_query(None);
        parsed
            .query_pairs_mut()
            .append_pair("ssl-mode", "REQUIRED");
        parsed.to_string()
    }
}
