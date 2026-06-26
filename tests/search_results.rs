use actix_web::{test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

fn handler_name(resp: &actix_web::dev::ServiceResponse) -> Option<String> {
    resp.headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

macro_rules! call {
    ($path:expr) => {{
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
        let req = test::TestRequest::get().uri($path).to_request();
        test::call_service(&app, req).await
    }};
}

#[actix_web::test]
async fn search_q_test_redirects_to_videos_test() {
    let resp = call!("/search?q=test");
    assert_eq!(resp.status(), 302);
    assert_eq!(handler_name(&resp).as_deref(), Some("search_redirect"));
    let loc = resp
        .headers()
        .get("location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        loc.starts_with("/videos/test"),
        "expected redirect to /videos/test, got {loc}"
    );
}

#[actix_web::test]
async fn videos_test_renders_title_h1_canonical_and_grid() {
    let resp = call!("/videos/test");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("videos_search"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);

    assert!(
        html.contains("Test Porn Videos &amp; Sex Scenes | PornsOK.com</title>")
            || html.contains("Test Porn Videos & Sex Scenes | PornsOK.com</title>"),
        "missing search title"
    );
    assert!(html.contains("<h1>Test Porn Videos</h1>"), "missing H1");
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/videos/test""#),
        "missing canonical"
    );
    assert!(html.contains("thumbs-floats"), "missing grid wrapper");
    assert!(html.contains("thumb vid"), "missing thumb class");
    assert!(
        html.contains("fa fa-eye") || html.contains("fa-eye"),
        "search thumbs should use eye icon for views"
    );
    assert!(
        html.contains("thumb-up-svg"),
        "search thumbs should use thumb-up SVG for likes"
    );
    assert!(
        html.contains("/video/site-test.html"),
        "expected fixture site-test video link"
    );
    assert!(html.contains("Relevant"), "missing Relevant sort label");
    assert!(!html.contains("stub:videos"), "search page still stub");
}

#[actix_web::test]
async fn videos_test_sort_recent_link_present() {
    let resp = call!("/videos/test");
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"href="/videos/test?sort=recent""#),
        "missing newest/recent sort link"
    );
}

#[actix_web::test]
async fn videos_test_page_two_uses_search_pagination_handler() {
    let resp = call!("/videos/test/2");
    let handler = handler_name(&resp);
    assert!(
        matches!(
            handler.as_deref(),
            Some("videos_search_page") | Some("not_found")
        ),
        "unexpected handler for /videos/test/2: {handler:?}"
    );
    if handler.as_deref() == Some("videos_search_page") {
        let body = test::read_body(resp).await;
        let html = String::from_utf8_lossy(&body);
        assert!(
            html.contains(r#"canonical" href="https://pornsok.com/videos/test/2""#)
                || html.contains("page_nav"),
            "page 2 should paginate when enough results"
        );
    }
}
