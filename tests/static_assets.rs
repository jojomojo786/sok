use actix_web::{http::StatusCode, test, web, App};
use regex::Regex;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers;
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

async fn assert_get_ok(path: &str) {
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
        "GET {path} should return 200, got {}",
        resp.status()
    );
}

async fn assert_page_assets_ok(page_path: &str) {
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

    let req = test::TestRequest::get().uri(page_path).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "GET {page_path} should return 200, got {}",
        resp.status()
    );

    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    let asset_paths = extract_local_asset_paths(&html);
    assert!(
        !asset_paths.is_empty(),
        "expected local asset references in {page_path}"
    );

    for path in asset_paths {
        let req = test::TestRequest::get().uri(&path).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "asset {path} referenced by {page_path} should return 200, got {}",
            resp.status()
        );
    }
}

fn extract_local_asset_paths(html: &str) -> Vec<String> {
    let attr_re = Regex::new(r#"(?:href|src|srcset|data-mobile)\s*=\s*["']([^"']+)["']"#)
        .expect("asset attribute regex");
    let mut paths = Vec::new();
    for cap in attr_re.captures_iter(html) {
        let raw = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        for candidate in raw.split_whitespace() {
            if let Some(path) = normalize_local_asset_path(candidate) {
                if !paths.contains(&path) {
                    paths.push(path);
                }
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

    if is_critical_asset_path(path) {
        Some(path.to_string())
    } else {
        None
    }
}

fn is_critical_asset_path(path: &str) -> bool {
    path.starts_with("/static/")
        || path.starts_with("/fox-tpl/")
        || matches!(
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

const CRITICAL_STATIC_PATHS: &[&str] = &[
    "/static/js/main.min.js",
    "/static/js/rocket-loader.min.js",
    "/static/fox-tpl/images/footer-logo.svg",
    "/static/fox-tpl/images/spacer.gif",
    "/static/fox-tpl/images/shadow.png",
    "/static/fox-tpl/images/loadMoreVideos.gif",
    "/static/fox-tpl/fonts/custom4/icomoon.woff",
    "/static/fox-tpl/js/playerjs.js",
    "/static/site.webmanifest",
    "/favicon.ico",
    "/apple-touch-icon.png",
    "/favicon-32x32.png",
    "/favicon-16x16.png",
    "/safari-pinned-tab.svg",
    "/android-chrome-192x192.png",
    "/android-chrome-512x512.png",
    "/fox-tpl/js/main.min.js",
    "/fox-tpl/js/rocket-loader.min.js",
    "/fox-tpl/images/footer-logo.svg",
    "/fox-tpl/images/spacer.gif",
    "/fox-tpl/images/shadow.png",
    "/fox-tpl/images/loadMoreVideos.gif",
    "/fox-tpl/fonts/custom4/icomoon.woff",
    "/fox-tpl/js/playerjs.js",
    "/site.webmanifest",
];

#[actix_web::test]
async fn critical_static_assets_return_200() {
    for path in CRITICAL_STATIC_PATHS {
        assert_get_ok(path).await;
    }
}

#[actix_web::test]
async fn home_page_static_js_and_footer_logo_return_200() {
    assert_get_ok("/static/js/main.min.js").await;
    assert_get_ok("/static/fox-tpl/images/footer-logo.svg").await;
}

#[actix_web::test]
async fn categories_style_fox_tpl_paths_return_200() {
    assert_get_ok("/fox-tpl/js/main.min.js").await;
    assert_get_ok("/fox-tpl/fonts/custom4/icomoon.ttf").await;
}

const KEMOJI_STATIC_PATHS: &[&str] = &[
    "/static/fox-tpl/js/smiles_.json",
    "/fox-tpl/js/smiles_.json",
    "/static/fox-tpl/style/img/opacity.png",
    "/static/fox-tpl/style/rez/back/emoji.png",
    "/static/fox-tpl/style/rez/bdsm/emoji.png",
    "/static/fox-tpl/style/rez/dick/emoji.png",
    "/static/fox-tpl/style/rez/dick2/emoji.png",
    "/static/fox-tpl/style/rez/front/emoji.png",
    "/static/fox-tpl/style/rez/one/emoji.png",
    "/static/fox-tpl/style/rez/pussy/emoji.png",
    "/static/fox-tpl/style/rez/pussy2/emoji.png",
    "/static/fox-tpl/style/rez/sperm/emoji.png",
    "/fox-tpl/style/rez/back/emoji.png",
    "/fox-tpl/style/rez/bdsm/emoji.png",
    "/fox-tpl/style/rez/dick/emoji.png",
    "/fox-tpl/style/rez/dick2/emoji.png",
    "/fox-tpl/style/rez/front/emoji.png",
    "/fox-tpl/style/rez/one/emoji.png",
    "/fox-tpl/style/rez/pussy/emoji.png",
    "/fox-tpl/style/rez/pussy2/emoji.png",
    "/fox-tpl/style/rez/sperm/emoji.png",
];

#[actix_web::test]
async fn kemoji_static_assets_return_200() {
    for path in KEMOJI_STATIC_PATHS {
        assert_get_ok(path).await;
    }
}

#[actix_web::test]
async fn home_page_referenced_local_assets_return_200() {
    assert_page_assets_ok("/").await;
}

#[actix_web::test]
async fn categories_page_referenced_local_assets_return_200() {
    assert_page_assets_ok("/categories").await;
}

#[actix_web::test]
async fn pornstars_page_referenced_local_assets_return_200() {
    assert_page_assets_ok("/pornstars").await;
}

#[actix_web::test]
async fn sample_video_page_referenced_local_assets_return_200() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    assert_page_assets_ok(&path).await;
}
