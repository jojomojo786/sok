use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::errors::not_found_page_response;
use crate::views::{legal_page_view, legal_static_context, LegalPageTemplate, SiteLayout};

use super::common::HANDLER_MARKER;

pub async fn page_static(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let layout = SiteLayout::production();
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
