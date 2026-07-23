use actix_web::{body::to_bytes, test, web, App};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

fn handler_name(resp: &actix_web::dev::ServiceResponse) -> Option<String> {
    resp.headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

macro_rules! call {
    ($method:ident, $path:expr $(, $payload:expr)?) => {{
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
        let req = test::TestRequest::$method()
            .uri($path)
            $(.insert_header(("Content-Type", "application/x-www-form-urlencoded"))
            .set_payload($payload))?
            .to_request();
        test::call_service(&app, req).await
    }};
}

async fn get_handler(path: &str) -> Option<String> {
    handler_name(&call!(get, path))
}

async fn get_html(path: &str) -> (actix_web::http::StatusCode, String, Option<String>) {
    let resp = call!(get, path);
    let status = resp.status();
    let handler = handler_name(&resp);
    let body = to_bytes(resp.into_body()).now_or_never().unwrap();
    let html = String::from_utf8(body.expect("response body").to_vec()).unwrap();
    (status, html, handler)
}

async fn assert_path_not_category_slug(path: &str) {
    let handler = get_handler(path).await;
    assert_ne!(
        handler.as_deref(),
        Some("category_slug"),
        "path {path} must not hit category slug fallback"
    );
}

#[actix_web::test]
async fn route_maps_index() {
    assert_eq!(get_handler("/").await.as_deref(), Some("index"));
}

#[actix_web::test]
async fn route_maps_categories() {
    assert_eq!(
        get_handler("/categories").await.as_deref(),
        Some("categories")
    );
}

#[actix_web::test]
async fn diagnostic_source_replay_is_disabled_by_default() {
    if std::env::var("SOK_DIAG_ROUTES").is_ok() {
        return;
    }

    let (status, _html, handler) = get_html("/_diag/source-replay/home").await;
    assert_eq!(status, 404);
    assert_eq!(handler.as_deref(), Some("diag_source_replay"));
}

#[actix_web::test]
async fn route_maps_pornstars() {
    assert_eq!(
        get_handler("/pornstars").await.as_deref(),
        Some("pornstars")
    );
}

#[actix_web::test]
async fn pornstars_returns_html_with_profile_links() {
    let (status, html, handler) = get_html("/pornstars").await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("pornstars"));
    assert!(html.contains("Best Pornstars and Models in Free Porn Videos"));
    assert!(html.contains("<h1>Top Trending Pornstars</h1>"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/pornstars""#));
    assert!(html.contains("/pornstar/"));
    assert!(!html.contains("stub:pornstars"));
}

#[actix_web::test]
async fn route_maps_channels() {
    assert_eq!(get_handler("/channels").await.as_deref(), Some("channels"));
}

#[actix_web::test]
async fn channels_returns_html_with_channel_links() {
    let (status, html, handler) = get_html("/channels").await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("channels"));
    assert!(html.contains("Top Trending Porn Channels"));
    assert!(html.contains("Free Porn Channels: List of Best Sex Channels"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/channels""#));
    assert!(html.contains("/channel/"));
    assert!(html.contains("thumb cat"));
    assert!(html.contains("#camera-svg"));
    assert!(!html.contains("stub:channels"));
}

#[actix_web::test]
async fn route_maps_tags() {
    assert_eq!(get_handler("/tags").await.as_deref(), Some("tags"));
}

#[actix_web::test]
async fn route_maps_home_numeric_pagination() {
    // Numeric home pages now render the dynamic home template when in range, or the
    // 404 page when the requested page exceeds the available data (fixture fallback).
    // Either outcome confirms the route maps to the home pagination handler rather
    // than the `/{slug}` category fallback.
    for path in ["/2", "/42"] {
        let handler = get_handler(path).await;
        assert!(
            matches!(
                handler.as_deref(),
                Some("home_page_num") | Some("not_found")
            ),
            "unexpected handler {handler:?} for {path}"
        );
    }
}

#[actix_web::test]
async fn route_maps_video_html() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    assert_eq!(get_handler(&path).await.as_deref(), Some("video_html"));
}

#[actix_web::test]
async fn video_sample_renders_player_shell_and_canonical() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    let (status, html, handler) = get_html(&path).await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("video_html"));
    assert!(html.contains("player_container2"));
    assert!(html.contains("v-meta"));
    assert!(html.contains(r#"rel="canonical" href="https://pornsok.com/video/dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do.html""#));
    assert!(!html.contains("stub:video"));
}

#[actix_web::test]
async fn unknown_video_slug_returns_styled_404() {
    let (status, html, handler) = get_html("/video/definitely-missing-slug-xyz.html").await;
    assert_eq!(status, 404);
    assert!(
        matches!(
            handler.as_deref(),
            Some("video_html") | Some("not_found") | None
        ),
        "unexpected handler {handler:?}"
    );
    assert!(html.contains("Page not found") || html.contains("footer"));
}

#[actix_web::test]
async fn route_maps_videofile_handler() {
    let path = "/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D";
    let handler = get_handler(path).await;
    assert!(
        matches!(handler.as_deref(), Some("videofile") | Some("not_found")),
        "unexpected handler {:?} for {path}",
        handler
    );
}

#[actix_web::test]
async fn route_maps_embeded_html_handler() {
    let path = "/embeded/example-slug.html";
    let handler = get_handler(path).await;
    assert!(
        matches!(handler.as_deref(), Some("embeded_html") | Some("not_found")),
        "unexpected handler {:?} for {path}",
        handler
    );
}

#[actix_web::test]
async fn route_maps_channel_profile() {
    assert_eq!(
        get_handler("/channel/brazzers").await.as_deref(),
        Some("channel_profile")
    );
}

#[actix_web::test]
async fn route_maps_pornstar_profile() {
    assert_eq!(
        get_handler("/pornstar/angela-white").await.as_deref(),
        Some("pornstar_profile")
    );
}

#[actix_web::test]
async fn route_maps_page_static_html() {
    assert_eq!(
        get_handler("/page/privacy.html").await.as_deref(),
        Some("page_static")
    );
}

#[actix_web::test]
async fn legal_page_sample_renders_meta_and_body_shell() {
    let (status, html, handler) = get_html("/page/privacy.html").await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("page_static"));
    assert!(html.contains("Privacy Policy - Pornsok.com</title>"));
    assert!(html.contains("<h1>Privacy Policy - PornsOK.COM</h1>"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/page/privacy.html""#));
    assert!(html.contains("class=\"desc-text page-text\""));
    assert!(!html.contains("stub:page"));
}

#[actix_web::test]
async fn unknown_legal_page_returns_styled_404() {
    let (status, html, handler) = get_html("/page/not-a-real-page.html").await;
    assert_eq!(status, 404);
    assert_eq!(handler.as_deref(), Some("page_static"));
    assert!(html.contains("Page not found") || html.contains("footer"));
}

#[actix_web::test]
async fn route_maps_search_redirect() {
    assert_eq!(
        get_handler("/search?q=test").await.as_deref(),
        Some("search_redirect")
    );
}

#[actix_web::test]
async fn route_maps_videos_search_page() {
    let handler = get_handler("/videos/test/2").await;
    assert_ne!(
        handler.as_deref(),
        Some("category_slug"),
        "/videos/{{query}}/{{page}} must not fall through to category slug"
    );
    assert!(
        matches!(
            handler.as_deref(),
            Some("videos_search_page") | Some("not_found")
        ),
        "unexpected handler {:?}",
        handler
    );
}

#[actix_web::test]
async fn route_maps_videos_search() {
    assert_eq!(
        get_handler("/videos/milf").await.as_deref(),
        Some("videos_search")
    );
}

#[actix_web::test]
async fn route_maps_category_slug_fallback() {
    assert_eq!(get_handler("/milf").await.as_deref(), Some("category_slug"));
}

#[actix_web::test]
async fn category_sample_renders_h1_grid_and_canonical() {
    let (status, html, handler) = get_html("/milf").await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("category_slug"));
    assert!(html.contains("<h1>MILF - Latest Porn Scenes</h1>"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/milf""#));
    assert!(html.contains("thumbs-floats"));
    assert!(html.contains("/video/"));
    assert!(!html.contains("stub:category"));
}

#[actix_web::test]
async fn unknown_category_slug_returns_styled_404() {
    let slug = format!(
        "no-such-taxonomy-slug-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let path = format!("/{slug}");
    let (status, html, handler) = get_html(&path).await;
    assert_eq!(status, 404);
    assert_eq!(handler.as_deref(), Some("category_slug"));
    assert!(html.contains("Page not found"));
    assert!(html.contains("footer"));
}

#[actix_web::test]
async fn post_ajax_search_help_sample_returns_json_contract() {
    let resp = call!(post, "/ajax/search_help", "text=milf");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("search_help"));
    // Matches live pornsok.com: JSON body served as `text/html` + `nosniff` so
    // the mirrored jQuery `$.parseJSON` path works (sok-replica.5.7).
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    let body = test::read_body(resp).await;
    let v: serde_json::Value = serde_json::from_slice(&body).expect("valid json");
    assert_eq!(v["search_text"].as_str(), Some("milf"));
    assert!(v["pornstars"].is_array());
    assert!(v["channels"].is_array());
    assert!(v["videos"].is_array());
}

#[actix_web::test]
async fn get_ajax_reserved_path_returns_stub_not_category_slug() {
    let resp = call!(get, "/ajax/unknown-endpoint");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("ajax"));
    let body = test::read_body(resp).await;
    assert_eq!(String::from_utf8_lossy(&body), "stub:ajax");
}

#[actix_web::test]
async fn post_unknown_ajax_search_type_returns_404() {
    let resp = call!(post, "/ajax/search_videos", "text=milf");
    assert_eq!(resp.status(), 404);
    assert_eq!(handler_name(&resp).as_deref(), Some("ajax"));
    let body = test::read_body(resp).await;
    assert_eq!(String::from_utf8_lossy(&body), "unsupported search type");
}

#[actix_web::test]
async fn reserved_paths_not_category_slug() {
    for path in [
        "/static",
        "/static/",
        "/fox-tpl",
        "/fox-tpl/",
        "/static/fox-tpl",
        "/static/fox-tpl/",
        "/site.webmanifest",
        "/ajax",
        "/ajax/search_help",
        "/ajax/more_videos_3",
        "/ajax/comments",
        "/ajax/more_comments",
        "/ajax/add_hit/favourite",
        "/ajax/add_hit/more_videos",
        "/ajax/add_vote_v3",
        "/ajax/search_pstars",
        "/ajax/search_channels",
        "/ajax/update_newest_videos",
        "/ajax/update_watching_now",
        "/ajax/update_tags",
        "/ajax/update_channels",
        "/ajax/update_pornstars",
        "/health",
    ] {
        assert_path_not_category_slug(path).await;
    }
}

#[actix_web::test]
async fn list_routes_precedence_over_slug_fallback() {
    for (path, expected) in [
        ("/pornstars", "pornstars"),
        ("/channels", "channels"),
        ("/tags", "tags"),
        ("/categories", "categories"),
    ] {
        assert_eq!(get_handler(path).await.as_deref(), Some(expected));
    }
}
