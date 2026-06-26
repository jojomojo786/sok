//! Manifest, favicon, and social metadata asset coverage for main pages.

use actix_web::{body::to_bytes, http::StatusCode, test, web, App};
use futures_util::future::FutureExt;
use regex::Regex;
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

const METADATA_HEAD_MARKERS: &[&str] = &[
    r##"<link rel="shortcut icon" href="/favicon.ico" type="image/x-icon">"##,
    r##"<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">"##,
    r##"<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png">"##,
    r##"<link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png">"##,
    r##"<link rel="manifest" href="/site.webmanifest">"##,
    r##"<link rel="mask-icon" href="/safari-pinned-tab.svg" color="#5bbad5">"##,
    r##"<meta name="msapplication-TileColor" content="#da532c">"##,
    r##"<meta name="theme-color" content="#ffffff">"##,
    r##"property="og:image" content="https://c.foxporn.tv/""##,
    r##"property="og:site_name" content="pornsok.com""##,
];

const ROOT_METADATA_ASSET_PATHS: &[&str] = &[
    "/favicon.ico",
    "/apple-touch-icon.png",
    "/favicon-32x32.png",
    "/favicon-16x16.png",
    "/safari-pinned-tab.svg",
    "/site.webmanifest",
    "/android-chrome-192x192.png",
    "/android-chrome-512x512.png",
];

const MAIN_PAGE_PATHS: &[&str] = &[
    "/",
    "/categories",
    "/milf",
    "/tags",
    "/pornstars",
    "/channels",
    "/pornstar/angela-white",
    "/channel/brazzers",
    "/videos/test",
    "/page/privacy.html",
];

async fn get_page(path: &str) -> (StatusCode, String, Option<String>) {
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

async fn assert_get_ok(path: &str, context: &str) {
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
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "{context}: GET {path} should return 200, got {}",
        resp.status()
    );
}

fn assert_metadata_head_tags(html: &str, family: &str) {
    for marker in METADATA_HEAD_MARKERS {
        assert!(
            html.contains(marker),
            "{family}: missing metadata marker `{marker}`"
        );
    }
    assert!(
        html.contains(r#"property="og:title""#) && html.contains(r#"property="og:url""#),
        "{family}: missing og:title or og:url"
    );
}

fn extract_head_metadata_asset_paths(html: &str) -> Vec<String> {
    let head = html
        .split_once("</head>")
        .map(|(head, _)| head)
        .unwrap_or(html);
    let href_re = Regex::new(r#"<link[^>]+href="([^"]+)""#).expect("link href regex");
    let mut paths = Vec::new();
    for cap in href_re.captures_iter(head) {
        let raw = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        if let Some(path) = normalize_local_asset_path(raw) {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

fn normalize_local_asset_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("data:")
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("//")
    {
        return None;
    }

    let path = trimmed.split('?').next().unwrap_or(trimmed);
    if !path.starts_with('/') {
        return None;
    }

    if is_metadata_asset_path(path) {
        Some(path.to_string())
    } else {
        None
    }
}

fn is_metadata_asset_path(path: &str) -> bool {
    matches!(
        path,
        "/favicon.ico"
            | "/apple-touch-icon.png"
            | "/favicon-32x32.png"
            | "/favicon-16x16.png"
            | "/safari-pinned-tab.svg"
            | "/site.webmanifest"
            | "/android-chrome-192x192.png"
            | "/android-chrome-512x512.png"
    )
}

async fn manifest_icon_paths() -> Vec<String> {
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
    let req = test::TestRequest::get()
        .uri("/site.webmanifest")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "site.webmanifest should return 200"
    );
    let body = test::read_body(resp).await;
    let manifest: Value = serde_json::from_slice(&body).expect("valid site.webmanifest json");
    assert_eq!(manifest["theme_color"].as_str(), Some("#ffffff"));
    assert_eq!(manifest["background_color"].as_str(), Some("#ffffff"));
    assert_eq!(manifest["display"].as_str(), Some("standalone"));

    let icons = manifest["icons"].as_array().expect("manifest icons array");
    let mut paths = Vec::new();
    for icon in icons {
        let src = icon["src"].as_str().expect("manifest icon src");
        if let Some(path) = normalize_local_asset_path(src) {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    paths
}

async fn assert_page_metadata_assets_ok(page_path: &str) {
    let (status, html, _) = get_page(page_path).await;
    assert_eq!(status, StatusCode::OK, "page {page_path} should return 200");
    assert_metadata_head_tags(&html, page_path);

    let mut asset_paths = extract_head_metadata_asset_paths(&html);
    for icon_path in manifest_icon_paths().await {
        if !asset_paths.contains(&icon_path) {
            asset_paths.push(icon_path);
        }
    }
    assert!(
        !asset_paths.is_empty(),
        "expected metadata asset references in {page_path}"
    );

    for path in asset_paths {
        assert_get_ok(&path, page_path).await;
    }
}

#[actix_web::test]
async fn root_metadata_assets_return_200() {
    for path in ROOT_METADATA_ASSET_PATHS {
        assert_get_ok(path, "root metadata assets").await;
    }
}

#[actix_web::test]
async fn site_webmanifest_icons_resolve() {
    let icon_paths = manifest_icon_paths().await;
    assert_eq!(
        icon_paths,
        vec![
            "/android-chrome-192x192.png".to_string(),
            "/android-chrome-512x512.png".to_string()
        ]
    );
    for path in icon_paths {
        assert_get_ok(&path, "site.webmanifest icons").await;
    }
}

#[actix_web::test]
async fn main_pages_metadata_head_tags_and_assets_resolve() {
    for page_path in MAIN_PAGE_PATHS {
        assert_page_metadata_assets_ok(page_path).await;
    }
}

#[actix_web::test]
async fn sample_video_page_metadata_head_tags_and_assets_resolve() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    assert_page_metadata_assets_ok(&path).await;
}
