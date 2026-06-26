use std::sync::Once;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,sok=debug,sqlx=warn,actix_server=info,actix_web=info")
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(true))
            .try_init()
            .ok();
    });
}

/// Redact credentials and connection URLs from driver messages before logging.
pub fn sanitize_db_error(err: &sqlx::Error) -> String {
    redact_sensitive_tokens(&err.to_string())
}

pub fn sanitize_message(msg: &str) -> String {
    redact_sensitive_tokens(msg)
}

fn redact_sensitive_tokens(raw: &str) -> String {
    let mut out = raw.to_string();

    for prefix in ["mysql://", "postgres://", "postgresql://"] {
        while let Some(start) = out.find(prefix) {
            let rest = &out[start..];
            let end = rest
                .find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ')')
                .unwrap_or(rest.len());
            out.replace_range(start..start + end, "<redacted-database-url>");
        }
    }

    for needle in ["password=", "passwd=", "pwd="] {
        if let Some(idx) = out.to_lowercase().find(needle) {
            let tail_start = idx + needle.len();
            let tail = &out[tail_start..];
            let end = tail
                .find(|c: char| c.is_whitespace() || c == '&' || c == ';')
                .unwrap_or(tail.len());
            out.replace_range(tail_start..tail_start + end, "<redacted>");
        }
    }

    out
}

pub fn log_db_pool_connecting(db_label: &str) {
    tracing::info!(db = db_label, "connecting to database");
}

pub fn log_db_pool_ready(db_label: &str) {
    tracing::info!(db = db_label, "database pool ready");
}

pub fn log_db_pool_failed(db_label: &str, err: &sqlx::Error) {
    tracing::error!(
        db = db_label,
        error = %sanitize_db_error(err),
        "database pool connection failed"
    );
}

pub fn log_request_db_error(handler: &'static str, err: &sqlx::Error) {
    tracing::error!(
        handler,
        error = %sanitize_db_error(err),
        "database error serving request"
    );
}

pub fn log_request_internal_error(handler: &'static str, detail: &str) {
    tracing::error!(
        handler,
        detail = %sanitize_message(detail),
        "internal error serving request"
    );
}

/// AJAX handlers that fall back to fixtures on DB failure (never log search/comment body text).
pub fn log_ajax_db_fallback(handler: &'static str, err: &sqlx::Error) {
    tracing::warn!(
        handler,
        error = %sanitize_db_error(err),
        "ajax database error; using fixture fallback"
    );
}

pub fn log_best_effort_db_skip(operation: &'static str, err: &sqlx::Error) {
    tracing::warn!(
        operation,
        error = %sanitize_db_error(err),
        "best-effort database operation skipped"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_db_error_redacts_mysql_url() {
        let err = sqlx::Error::Configuration(
            "mysql://secret_user:super_secret_pw@db.internal:3306/sok".into(),
        );
        let sanitized = sanitize_db_error(&err);
        assert!(!sanitized.contains("super_secret_pw"));
        assert!(!sanitized.contains("secret_user"));
        assert!(sanitized.contains("<redacted-database-url>"));
    }

    #[test]
    fn sanitize_message_redacts_password_query_param() {
        let msg = "connect failed password=hunter2 host=db";
        let sanitized = sanitize_message(msg);
        assert!(!sanitized.contains("hunter2"));
        assert!(sanitized.contains("password=<redacted>"));
    }
}
