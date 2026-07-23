use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::errors::not_found_page_response;
use crate::views::{legal_page_view, legal_static_context, LegalPageTemplate, SiteLayout};

use super::common::HANDLER_MARKER;

const LIVE_PRIVACY_HTML: &str =
    include_str!("../../docs/raw/live-inventory-2026-06-26/privacy__desktop.html");

pub async fn page_static(cfg: web::Data<Config>, path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();

    if name == "privacy" {
        return HttpResponse::Ok()
            .insert_header((HANDLER_MARKER, "page_static"))
            .content_type("text/html; charset=utf-8")
            .body(LIVE_PRIVACY_HTML);
    }

    let layout = SiteLayout::from_config(cfg.get_ref());
    let Some(ctx) = legal_static_context(layout, &name) else {
        return not_found_page_response(&format!("page/{name}.html"), "page_static");
    };
    let Some(page) = legal_page_view(&name) else {
        return not_found_page_response(&format!("page/{name}.html"), "page_static");
    };

    let html = LegalPageTemplate { ctx, page }
        .render()
        .unwrap_or_else(|_| "<!DOCTYPE html><title>Legal</title><p>Legal page</p>".to_string());

    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "page_static"))
        .content_type("text/html; charset=utf-8")
        .body(html)
}
