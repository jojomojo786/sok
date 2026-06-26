use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use askama::Template;
use std::fmt;

use crate::logging::{log_request_db_error, log_request_internal_error};
use crate::views::{ErrorTemplate, RenderContext, SiteLayout};

/// User-visible copy for unexpected failures (must not echo DB/driver details).
pub const PUBLIC_INTERNAL_ERROR_MESSAGE: &str =
    "Something went wrong on our side. Please try again in a moment.";

#[derive(Debug)]
pub enum AppError {
    Db(sqlx::Error),
    Internal(String),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Db(e) => write!(f, "Database error: {e}"),
            AppError::Internal(msg) => write!(f, "Internal error: {msg}"),
            AppError::NotFound(msg) => write!(f, "Not found: {msg}"),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Db(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::NotFound(slug) => not_found_page_response(slug, "not_found"),
            AppError::Db(e) => {
                log_request_db_error("internal_error", &e);
                let ctx = RenderContext::internal_error_page(SiteLayout::production());
                render_error_html(ctx)
            }
            AppError::Internal(msg) => {
                log_request_internal_error("internal_error", &msg);
                let ctx = RenderContext::internal_error_page(SiteLayout::production());
                render_error_html(ctx)
            }
        }
    }
}

pub fn not_found_page_response(slug: &str, handler_marker: &'static str) -> HttpResponse {
    let ctx = RenderContext::not_found_page(SiteLayout::production(), slug);
    let html = ErrorTemplate { ctx }
        .render()
        .unwrap_or_else(|_| "<!DOCTYPE html><title>404</title><p>Page not found</p>".to_string());
    HttpResponse::build(StatusCode::NOT_FOUND)
        .insert_header(("X-Sok-Handler", handler_marker))
        .content_type("text/html; charset=utf-8")
        .body(html)
}

fn render_error_html(ctx: RenderContext) -> HttpResponse {
    let html = ErrorTemplate { ctx }
        .render()
        .unwrap_or_else(|_| "<!DOCTYPE html><title>Error</title><p>Error</p>".to_string());
    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .insert_header(("X-Sok-Handler", "internal_error"))
        .content_type("text/html; charset=utf-8")
        .body(html)
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::future::FutureExt;

    #[test]
    fn db_error_response_does_not_leak_driver_message() {
        let err = AppError::Db(sqlx::Error::Configuration(
            "mysql://secret_user:super_secret_pw@db.internal:3306/sok".into(),
        ));
        let body = err.error_response().into_body();
        let bytes = actix_web::body::to_bytes(body)
            .now_or_never()
            .unwrap()
            .expect("response body");
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(!text.contains("super_secret_pw"));
        assert!(!text.contains("secret_user"));
        assert!(!text.contains("mysql://"));
        assert!(text.contains(PUBLIC_INTERNAL_ERROR_MESSAGE));
    }

    #[test]
    fn not_found_returns_404_html_shell() {
        let err = AppError::NotFound("definitely-missing-slug".into());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let bytes = actix_web::body::to_bytes(resp.into_body())
            .now_or_never()
            .unwrap()
            .expect("response body");
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(text.contains("Page not found") || text.contains("404"));
        assert!(text.contains("footer") || text.contains("PornsOK"));
    }
}
