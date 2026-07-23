use actix_web::{web, HttpResponse, Responder};

use crate::errors::not_found_page_response;

use super::common::HANDLER_MARKER;

const HOME_HTML: &str = include_str!("../../docs/raw/live-source-2026-06-27/home.html");
const CATEGORIES_HTML: &str = include_str!("../../docs/raw/live-source-2026-06-27/categories.html");
const PORNSTARS_HTML: &str = include_str!("../../docs/raw/live-source-2026-06-27/pornstars.html");
const CHANNELS_HTML: &str = include_str!("../../docs/raw/live-source-2026-06-27/channels.html");
const PRIVACY_HTML: &str = include_str!("../../docs/raw/live-source-2026-06-27/privacy.html");
const UPDATE_PORNSTARS_HTML: &str = include_str!("../../docs/raw/update_pornstars.body");
const UPDATE_CHANNELS_HTML: &str = include_str!("../../docs/raw/update_channels.body");

pub async fn source_replay(path: web::Path<String>) -> impl Responder {
    let label = path.into_inner();
    captured_html(&label, "diag_source_replay").unwrap_or_else(|| {
        not_found_page_response(
            &format!("_diag/source-replay/{label}"),
            "diag_source_replay",
        )
    })
}

pub async fn source_replay_page(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let label = match name.as_str() {
        "privacy" => "privacy",
        _ => {
            return not_found_page_response(
                &format!("_diag/source-replay/page/{name}.html"),
                "diag_source_replay",
            )
        }
    };
    captured_html(label, "diag_source_replay").unwrap_or_else(|| {
        not_found_page_response(
            &format!("_diag/source-replay/page/{name}.html"),
            "diag_source_replay",
        )
    })
}

pub async fn source_replay_ajax(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    captured_ajax_html(&name, "diag_source_replay").unwrap_or_else(|| {
        not_found_page_response(
            &format!("_diag/source-replay/ajax/{name}"),
            "diag_source_replay",
        )
    })
}

fn captured_html(label: &str, marker: &'static str) -> Option<HttpResponse> {
    if !diag_routes_enabled() {
        return None;
    }
    let body = match label {
        "home" => HOME_HTML,
        "categories" => CATEGORIES_HTML,
        "pornstars" => PORNSTARS_HTML,
        "channels" => CHANNELS_HTML,
        "privacy" => PRIVACY_HTML,
        _ => return None,
    };
    Some(
        HttpResponse::Ok()
            .insert_header((HANDLER_MARKER, marker))
            .content_type("text/html; charset=utf-8")
            .body(body),
    )
}

fn captured_ajax_html(label: &str, marker: &'static str) -> Option<HttpResponse> {
    if !diag_routes_enabled() {
        return None;
    }
    let body = match label {
        "update_pornstars" => UPDATE_PORNSTARS_HTML,
        "update_channels" => UPDATE_CHANNELS_HTML,
        _ => return None,
    };
    Some(
        HttpResponse::Ok()
            .insert_header((HANDLER_MARKER, marker))
            .insert_header(("Content-Type", "text/html; charset=UTF-8"))
            .body(body),
    )
}

fn diag_routes_enabled() -> bool {
    matches!(
        std::env::var("SOK_DIAG_ROUTES").ok().as_deref(),
        Some("1" | "true" | "TRUE" | "yes" | "YES")
    )
}
