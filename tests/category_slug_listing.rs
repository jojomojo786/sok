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
async fn milf_listing_renders_h1_grid_and_canonical() {
    let resp = call!("/milf");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("category_slug"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<h1>MILF - Latest Porn Scenes</h1>"),
        "missing H1 pattern"
    );
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/milf""#),
        "missing canonical url"
    );
    assert!(html.contains("thumbs-floats"), "missing video grid wrapper");
    assert!(html.contains("/video/"), "missing video links");
}

#[actix_web::test]
async fn milf_page_two_paginates_without_home_collision() {
    let resp = call!("/milf/2");
    let handler = handler_name(&resp);
    assert!(
        matches!(
            handler.as_deref(),
            Some("category_slug") | Some("not_found")
        ),
        "unexpected handler {handler:?} for /milf/2"
    );

    let resp = call!("/2");
    let home_handler = handler_name(&resp);
    // `/2` must resolve through the home numeric pagination handler (rendering the
    // dynamic page when in range, or the 404 page when out of range), never the
    // `/{slug}` category fallback.
    assert!(
        matches!(
            home_handler.as_deref(),
            Some("home_page_num") | Some("not_found")
        ),
        "/2 must remain home numeric pagination, got {home_handler:?}"
    );
    assert_ne!(home_handler.as_deref(), Some("category_slug"));
}

#[actix_web::test]
async fn tags_hub_renders_real_page() {
    let resp = call!("/tags");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("tags"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<h1>Porn Video Tags</h1>"),
        "missing tags hub H1"
    );
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/tags""#),
        "missing tags canonical"
    );
    assert!(!html.contains("stub:tags"), "tags hub is still a stub");
}

#[actix_web::test]
async fn unknown_slug_returns_404() {
    let resp = call!("/this-slug-definitely-does-not-exist-xyz");
    assert_eq!(resp.status(), 404);
    assert_eq!(handler_name(&resp).as_deref(), Some("category_slug"));
}
