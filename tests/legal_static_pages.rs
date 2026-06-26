use actix_web::{body::to_bytes, test, web, App};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn get_html(path: &str) -> (actix_web::http::StatusCode, String, Option<String>) {
    dotenv::dotenv().ok();
    let cfg = Config::load().expect("config");
    let pool = db::create_pool(&cfg).await;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .app_data(web::Data::new(cfg))
            .configure(configure_static)
            .configure(handlers::routes),
    )
    .await;
    let req = test::TestRequest::get().uri(path).to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let handler = resp
        .headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = to_bytes(resp.into_body()).now_or_never().unwrap();
    let html = String::from_utf8(body.expect("response body").to_vec()).unwrap();
    (status, html, handler)
}

fn assert_footer_nofollow(html: &str) {
    for href in [
        "/page/privacy.html",
        "/page/dmca.html",
        "/page/terms.html",
        "/page/2557.html",
        "/page/contact.html",
    ] {
        assert!(
            html.contains(&format!(r#"href="{href}" rel="nofollow""#)),
            "missing nofollow footer link for {href}"
        );
    }
}

#[actix_web::test]
async fn legal_pages_return_200_with_meta_and_footer() {
    let cases = [
        (
            "/page/privacy.html",
            "Privacy Policy - Pornsok.com</title>",
            "<h1>Privacy Policy - PornsOK.COM</h1>",
            r#"canonical" href="https://pornsok.com/page/privacy.html""#,
        ),
        (
            "/page/dmca.html",
            "DMCA - Pornsok.com</title>",
            "<h1>DMCA</h1>",
            r#"canonical" href="https://pornsok.com/page/dmca.html""#,
        ),
        (
            "/page/terms.html",
            "Terms - Pornsok.com</title>",
            "<h1>Terms</h1>",
            r#"canonical" href="https://pornsok.com/page/terms.html""#,
        ),
        (
            "/page/2557.html",
            "18 U.S.C. 2257 - Pornsok.com</title>",
            "<h1>18 U.S.C. 2257</h1>",
            r#"canonical" href="https://pornsok.com/page/2557.html""#,
        ),
        (
            "/page/contact.html",
            "Contact - Pornsok.com</title>",
            "<h1>Contact</h1>",
            r#"canonical" href="https://pornsok.com/page/contact.html""#,
        ),
    ];

    for (path, title_needle, h1_needle, canonical_needle) in cases {
        let (status, html, handler) = get_html(path).await;
        assert_eq!(status, 200, "status for {path}");
        assert_eq!(
            handler.as_deref(),
            Some("page_static"),
            "handler for {path}"
        );
        assert!(html.contains(title_needle), "title for {path}");
        assert!(html.contains(h1_needle), "h1 for {path}");
        assert!(html.contains(canonical_needle), "canonical for {path}");
        assert!(
            html.contains("class=\"desc-text page-text\""),
            "body shell for {path}"
        );
        assert_footer_nofollow(&html);
        assert!(!html.contains("stub:page"));
    }
}

#[actix_web::test]
async fn unknown_legal_page_returns_styled_404() {
    let (status, html, handler) = get_html("/page/not-a-real-page.html").await;
    assert_eq!(status, 404);
    assert_eq!(handler.as_deref(), Some("page_static"));
    assert!(html.contains("Page not found") || html.contains("footer"));
}
