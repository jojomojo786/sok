use std::fs;
use std::path::PathBuf;

use actix_web::{http::StatusCode, test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers;

const THUMB_LAZY_PLACEHOLDER: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

async fn fetch_html(path: &str) -> String {
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
    let body = test::read_body(resp).await;
    String::from_utf8_lossy(&body).into_owned()
}

fn main_min_js_contract() -> String {
    let candidates = [
        PathBuf::from("static/js/main.min.js"),
        PathBuf::from("static/fox-tpl/js/main.min.js"),
    ];
    for path in candidates {
        if path.is_file() {
            return fs::read_to_string(&path).expect("read main.min.js");
        }
    }
    panic!("main.min.js not found under static/js or static/fox-tpl/js");
}

fn assert_home_lazy_markup(html: &str) {
    assert!(
        html.contains("class=\"thumb-cover\"") && html.contains("data-original="),
        "home grid should render lazy-load poster hooks"
    );
    assert!(
        html.contains("data-video=") && html.contains("video-preview"),
        "home grid should render hover preview hooks"
    );
    assert!(
        html.contains(THUMB_LAZY_PLACEHOLDER),
        "home grid should keep the 1x1 GIF placeholder src"
    );
    assert!(
        html.contains("isTHUMBS_OR_PLAYER = true")
            && html.contains("directory = \"/static/fox-tpl\"")
            && html.contains("lazyThreshold = 2000"),
        "home boot globals should enable thumb preview behavior"
    );
    assert!(
        html.contains("src=\"/static/js/main.min.js\""),
        "home should load the local main.min.js bundle"
    );
}

fn assert_main_min_js_lazy_hover_contract(js: &str) {
    assert!(
        js.contains("myLazyLoad=new LazyLoad")
            && js.contains("elements_selector:\".thumb-cover, .ke, .soc-img\"")
            && js.contains("data_src:\"original\""),
        "main.min.js should initialize LazyLoad against .thumb-cover data-original"
    );
    assert!(
        js.contains("mouseenter\",\".video-preview\"")
            && js.contains("video-preview__video")
            && js.contains("getAttribute(\"data-video\")"),
        "main.min.js should inject hover preview videos from data-video"
    );
    assert!(
        js.contains("ontouchstart\"in window")
            && js.contains("touchstart")
            && js.contains("classList.contains(\"video-preview\")"),
        "main.min.js should provide mobile touch preview fallback"
    );
}

#[actix_web::test]
async fn home_page_emits_lazy_load_and_hover_preview_hooks() {
    let html = fetch_html("/").await;
    assert_home_lazy_markup(&html);

    let js = main_min_js_contract();
    assert_main_min_js_lazy_hover_contract(&js);
}

#[actix_web::test]
async fn update_watching_now_fragment_keeps_preview_hooks_for_ajax_cards() {
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

    let req = test::TestRequest::post()
        .uri("/ajax/update_watching_now")
        .set_form(&[("order_by", "week_views")])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);

    assert!(html.contains(r#"class="thumb vid""#));
    assert!(html.contains("video-preview"));
    assert!(html.contains("data-video="));
    assert!(html.contains("thumb-cover"));
}
