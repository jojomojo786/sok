use actix_web::HttpResponse;

/// Response marker used by route precedence tests to identify the matched handler.
pub const HANDLER_MARKER: &str = "X-Sok-Handler";

pub(crate) fn stub_response(handler: &'static str) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, handler))
        .content_type("text/plain; charset=utf-8")
        .body(format!("stub:{handler}"))
}
