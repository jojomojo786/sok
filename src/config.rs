use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::path::Path;

use serde::Deserialize;
use url::Url;

pub const CONFIG_JSON_PATH: &str = "config.json";
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

pub const DEFAULT_MEDIA_CDN: &str = "https://c.foxporn.tv";
pub const DEFAULT_STATIC_ROOT: &str = "/static";
pub const DEFAULT_FOX_TPL_ROOT: &str = "/fox-tpl";
pub const DEFAULT_THUMBS_VIDEOS_DIR: &str = "fox-images/videos";
pub const DEFAULT_VIDEO_PATH_SEGMENT: &str = "video";

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    /// CDN host for adult catalog media (thumbs, previews, entity art).
    pub media_cdn: String,
    /// Local mount for app static assets (`/static`).
    pub static_root: String,
    /// Local mount for mirrored fox-tpl assets (`/fox-tpl`).
    pub fox_tpl_root: String,
    /// CDN path segment for video thumb/preview MP4s (appended to `media_cdn`).
    pub thumbs_videos_dir: String,
    /// CDN path segment for full-video download placeholders.
    pub video_path_segment: String,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingDatabaseUrl,
    MissingDatabasePassword,
    InvalidDatabaseUrl(String),
    InvalidBindAddr {
        value: String,
        reason: String,
    },
    ConfigFileRead {
        path: String,
        source: std::io::Error,
    },
    ConfigFileParse {
        path: String,
        source: serde_json::Error,
    },
    InvalidMediaCdn {
        value: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingDatabaseUrl => write!(
                f,
                "Database connection is not configured. Set DATABASE_URL in .env (see .env.example), \
                 or set db.password in {CONFIG_JSON_PATH}, or export DB_PASSWORD when {CONFIG_JSON_PATH} leaves db.password empty."
            ),
            ConfigError::MissingDatabasePassword => write!(
                f,
                "Database password is missing. Put credentials in DATABASE_URL via .env (see .env.example), \
                 set db.password in {CONFIG_JSON_PATH}, or export DB_PASSWORD for local/dev overrides."
            ),
            ConfigError::InvalidDatabaseUrl(reason) => {
                write!(f, "DATABASE_URL is invalid: {reason}")
            }
            ConfigError::InvalidBindAddr { value, reason } => write!(
                f,
                "BIND_ADDR must be a valid socket address (example: {DEFAULT_BIND_ADDR}); got `{value}`: {reason}"
            ),
            ConfigError::ConfigFileRead { path, source } => {
                write!(f, "Failed to read config file `{path}`: {source}")
            }
            ConfigError::ConfigFileParse { path, source } => {
                write!(f, "Failed to parse config file `{path}` as JSON: {source}")
            }
            ConfigError::InvalidMediaCdn { value } => write!(
                f,
                "MEDIA_CDN must be an http(s) URL (example: {DEFAULT_MEDIA_CDN}); got `{value}`"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Loads runtime config from environment variables with optional `config.json` fallback.
    ///
    /// Precedence:
    /// - `DATABASE_URL` overrides database settings from `config.json`
    /// - `BIND_ADDR` overrides the default listen address (`0.0.0.0:8080`)
    /// - `DB_PASSWORD` can supply the password when `config.json` has an empty `db.password`
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_env_and_path(std::env::vars().collect(), Path::new(CONFIG_JSON_PATH))
    }

    pub(crate) fn load_from_env_and_path(
        env: HashMap<String, String>,
        config_path: &Path,
    ) -> Result<Self, ConfigError> {
        let bind_addr = resolve_bind_addr(&env)?;
        let database_url = resolve_database_url(&env, config_path)?;
        let media_cdn = resolve_media_cdn(&env)?;
        let static_root = resolve_static_root(&env);
        let fox_tpl_root = resolve_fox_tpl_root(&env);
        let thumbs_videos_dir = resolve_thumbs_videos_dir(&env);
        let video_path_segment = resolve_video_path_segment(&env);

        Ok(Config {
            database_url: normalize_mysql_url(&database_url)?,
            bind_addr,
            media_cdn,
            static_root,
            fox_tpl_root,
            thumbs_videos_dir,
            video_path_segment,
        })
    }

    /// CDN URL prefix for video thumb/preview assets.
    pub fn thumbs_videos_url(&self) -> String {
        media_url(&self.media_cdn, &self.thumbs_videos_dir)
    }

    /// Asset/CDN defaults used by template render contexts and tests.
    pub fn asset_defaults() -> Self {
        Self {
            database_url: String::new(),
            bind_addr: DEFAULT_BIND_ADDR.to_string(),
            media_cdn: DEFAULT_MEDIA_CDN.to_string(),
            static_root: DEFAULT_STATIC_ROOT.to_string(),
            fox_tpl_root: DEFAULT_FOX_TPL_ROOT.to_string(),
            thumbs_videos_dir: DEFAULT_THUMBS_VIDEOS_DIR.to_string(),
            video_path_segment: DEFAULT_VIDEO_PATH_SEGMENT.to_string(),
        }
    }

    /// Safe database label for logs (no password).
    pub fn database_log_label(&self) -> String {
        let parsed = match Url::parse(&self.database_url) {
            Ok(url) => url,
            Err(_) => return "mysql://<invalid-url>".to_string(),
        };
        let host = parsed.host_str().unwrap_or("unknown-host");
        let port = parsed.port().unwrap_or(3306);
        let user = if parsed.username().is_empty() {
            "unknown-user"
        } else {
            parsed.username()
        };
        let db = parsed.path().trim_start_matches('/');
        let db = if db.is_empty() { "unknown-db" } else { db };
        format!("mysql://{user}@{host}:{port}/{db}")
    }
}

fn env_value<'a>(env: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    env.get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn resolve_bind_addr(env: &HashMap<String, String>) -> Result<String, ConfigError> {
    let raw = env_value(env, "BIND_ADDR").unwrap_or(DEFAULT_BIND_ADDR);
    raw.parse::<SocketAddr>()
        .map_err(|err| ConfigError::InvalidBindAddr {
            value: raw.to_string(),
            reason: err.to_string(),
        })?;
    Ok(raw.to_string())
}

fn resolve_media_cdn(env: &HashMap<String, String>) -> Result<String, ConfigError> {
    let raw = env_value(env, "MEDIA_CDN").unwrap_or(DEFAULT_MEDIA_CDN);
    normalize_media_cdn(raw)
}

fn resolve_static_root(env: &HashMap<String, String>) -> String {
    env_value(env, "STATIC_ROOT")
        .unwrap_or(DEFAULT_STATIC_ROOT)
        .trim_end_matches('/')
        .to_string()
}

fn resolve_fox_tpl_root(env: &HashMap<String, String>) -> String {
    env_value(env, "FOX_TPL_ROOT")
        .unwrap_or(DEFAULT_FOX_TPL_ROOT)
        .trim_end_matches('/')
        .to_string()
}

fn resolve_thumbs_videos_dir(env: &HashMap<String, String>) -> String {
    env_value(env, "THUMBS_VIDEOS_DIR")
        .unwrap_or(DEFAULT_THUMBS_VIDEOS_DIR)
        .trim_matches('/')
        .to_string()
}

fn resolve_video_path_segment(env: &HashMap<String, String>) -> String {
    env_value(env, "VIDEO_PATH_SEGMENT")
        .unwrap_or(DEFAULT_VIDEO_PATH_SEGMENT)
        .trim_matches('/')
        .to_string()
}

fn normalize_media_cdn(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim().trim_end_matches('/');
    let parsed = Url::parse(trimmed).map_err(|_| ConfigError::InvalidMediaCdn {
        value: raw.to_string(),
    })?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(ConfigError::InvalidMediaCdn {
            value: raw.to_string(),
        });
    }
    if parsed.host().is_none() {
        return Err(ConfigError::InvalidMediaCdn {
            value: raw.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

pub fn media_url(cdn_base: &str, path: &str) -> String {
    let base = cdn_base.trim().trim_end_matches('/');
    let segment = path.trim().trim_matches('/');
    if segment.is_empty() {
        return base.to_string();
    }
    format!("{base}/{segment}")
}

fn resolve_database_url(
    env: &HashMap<String, String>,
    config_path: &Path,
) -> Result<String, ConfigError> {
    if let Some(url) = env_value(env, "DATABASE_URL") {
        return Ok(url.to_string());
    }

    match database_url_from_config_file(env, config_path)? {
        Some(url) => Ok(url),
        None if config_path.exists() => Err(ConfigError::MissingDatabasePassword),
        None => Err(ConfigError::MissingDatabaseUrl),
    }
}

fn database_url_from_config_file(
    env: &HashMap<String, String>,
    config_path: &Path,
) -> Result<Option<String>, ConfigError> {
    #[derive(Deserialize)]
    struct DbConfig {
        host: String,
        port: u16,
        database: String,
        user: String,
        #[serde(default)]
        password: String,
    }
    #[derive(Deserialize)]
    struct ConfigFile {
        db: DbConfig,
    }

    let file = match std::fs::read_to_string(config_path) {
        Ok(contents) => contents,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ConfigError::ConfigFileRead {
                path: config_path.display().to_string(),
                source,
            });
        }
    };

    let cf: ConfigFile =
        serde_json::from_str(&file).map_err(|source| ConfigError::ConfigFileParse {
            path: config_path.display().to_string(),
            source,
        })?;

    let password = if !cf.db.password.is_empty() {
        cf.db.password
    } else {
        match env.get("DB_PASSWORD").map(String::as_str) {
            Some(value) if !value.trim().is_empty() => value.to_string(),
            _ => return Ok(None),
        }
    };

    Ok(Some(format!(
        "mysql://{}:{}@{}:{}/{}",
        urlencoding::encode(&cf.db.user),
        urlencoding::encode(&password),
        cf.db.host,
        cf.db.port,
        cf.db.database
    )))
}

fn normalize_mysql_url(raw: &str) -> Result<String, ConfigError> {
    let mut parsed = Url::parse(raw)
        .map_err(|err| ConfigError::InvalidDatabaseUrl(format!("must be a valid URL ({err})")))?;
    if parsed.scheme() != "mysql" {
        return Err(ConfigError::InvalidDatabaseUrl(
            "must use the mysql:// scheme".into(),
        ));
    }
    if parsed.host().is_none() {
        return Err(ConfigError::InvalidDatabaseUrl(
            "must include a database host".into(),
        ));
    }
    if parsed.username().is_empty() {
        return Err(ConfigError::InvalidDatabaseUrl(
            "must include a database user".into(),
        ));
    }
    parsed.set_query(None);
    parsed.query_pairs_mut().append_pair("ssl-mode", "REQUIRED");
    Ok(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn lock_env() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().expect("env test lock")
    }

    fn with_temp_config<F>(contents: &str, run: F)
    where
        F: FnOnce(&Path),
    {
        let _guard = lock_env();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("sok-config-test-{nonce}.json"));
        fs::write(&path, contents).expect("write temp config");
        run(&path);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn load_prefers_database_url_over_config_json() {
        let env = HashMap::from([
            (
                "DATABASE_URL".into(),
                "mysql://env_user:env_pass@db.internal:3306/sok".into(),
            ),
            ("BIND_ADDR".into(), "127.0.0.1:9090".into()),
        ]);

        with_temp_config(
            r#"{"db":{"host":"ignored","port":1,"database":"ignored","user":"ignored","password":"ignored"}}"#,
            |path| {
                let cfg = Config::load_from_env_and_path(env.clone(), path).expect("load config");
                assert_eq!(cfg.bind_addr, "127.0.0.1:9090");
                assert!(cfg.database_url.starts_with("mysql://env_user:"));
                assert!(cfg.database_url.contains("ssl-mode=REQUIRED"));
                assert!(cfg.database_url.contains("@db.internal:3306/sok"));
            },
        );
    }

    #[test]
    fn config_json_password_can_come_from_db_password_env() {
        let env = HashMap::from([("DB_PASSWORD".into(), "from_env_secret".into())]);

        with_temp_config(
            r#"{"db":{"host":"db.internal","port":22451,"database":"sok","user":"avnadmin","password":""}}"#,
            |path| {
                let cfg = Config::load_from_env_and_path(env, path).expect("load config");
                assert_eq!(cfg.bind_addr, DEFAULT_BIND_ADDR);
                assert!(cfg.database_url.contains("avnadmin"));
                assert!(cfg.database_url.contains("db.internal:22451/sok"));
                assert!(cfg.database_url.contains("from_env_secret"));
            },
        );
    }

    #[test]
    fn missing_database_url_is_actionable_without_secrets() {
        let env = HashMap::new();

        with_temp_config(
            r#"{"db":{"host":"db.internal","port":22451,"database":"sok","user":"avnadmin","password":""}}"#,
            |path| {
                let err = Config::load_from_env_and_path(env, path).unwrap_err();
                let message = err.to_string();
                assert!(message.contains("DATABASE_URL"));
                assert!(message.contains(".env.example"));
                assert!(message.contains("DB_PASSWORD"));
            },
        );
    }

    #[test]
    fn invalid_bind_addr_is_actionable() {
        let env = HashMap::from([
            (
                "DATABASE_URL".into(),
                "mysql://user:pass@db.internal:3306/sok".into(),
            ),
            ("BIND_ADDR".into(), "not-a-socket".into()),
        ]);

        let err =
            Config::load_from_env_and_path(env, Path::new("missing-config.json")).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("BIND_ADDR"));
        assert!(message.contains("not-a-socket"));
        assert!(!message.contains("pass"));
    }

    #[test]
    fn invalid_database_url_does_not_leak_password() {
        let err =
            normalize_mysql_url("postgres://secret_user:super_secret_pw@db.internal:5432/sok")
                .unwrap_err();
        let message = err.to_string();
        assert!(message.contains("mysql://"));
        assert!(!message.contains("super_secret_pw"));
        assert!(!message.contains("secret_user"));
    }

    #[test]
    fn media_cdn_env_overrides_default() {
        let env = HashMap::from([
            (
                "DATABASE_URL".into(),
                "mysql://user:pass@db.internal:3306/sok".into(),
            ),
            ("MEDIA_CDN".into(), "https://cdn.example.com/".into()),
        ]);
        let cfg =
            Config::load_from_env_and_path(env, Path::new("missing-config.json")).expect("load");
        assert_eq!(cfg.media_cdn, "https://cdn.example.com");
        assert_eq!(
            cfg.thumbs_videos_url(),
            "https://cdn.example.com/fox-images/videos"
        );
    }

    #[test]
    fn invalid_media_cdn_is_actionable() {
        let env = HashMap::from([
            (
                "DATABASE_URL".into(),
                "mysql://user:pass@db.internal:3306/sok".into(),
            ),
            ("MEDIA_CDN".into(), "not-a-url".into()),
        ]);
        let err =
            Config::load_from_env_and_path(env, Path::new("missing-config.json")).unwrap_err();
        assert!(err.to_string().contains("MEDIA_CDN"));
    }

    #[test]
    fn load_reads_dotenv_when_database_url_present() {
        let _guard = lock_env();
        dotenv::dotenv().ok();

        if std::env::var("DATABASE_URL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .is_none()
        {
            return;
        }

        let cfg = Config::load().expect("local config from .env");
        assert!(cfg.database_url.starts_with("mysql://"));
        assert!(cfg.database_url.contains("ssl-mode=REQUIRED"));
        assert!(!cfg.bind_addr.is_empty());
    }
}
