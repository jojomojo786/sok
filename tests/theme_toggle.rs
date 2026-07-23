use std::fs;
use std::path::PathBuf;

use actix_web::{http::StatusCode, test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers;
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

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

fn assert_theme_page_contract(html: &str, path: &str) {
    assert!(
        html.contains("id=\"sun-svg\"") && html.contains("id=\"moon-svg\""),
        "{path}: missing inline sun/moon SVG symbol defs"
    );
    assert!(
        html.contains(":root") && html.contains("[data-theme=\"dark\"]"),
        "{path}: missing CSS variable theme blocks"
    );
    assert!(
        html.contains("#day-night-icon.to-day") && html.contains("#day-night-icon.to-night"),
        "{path}: missing icon fill hooks for day/night classes"
    );
    assert!(
        html.contains("class=\"btn-mob\""),
        "{path}: missing mobile menu toggle control"
    );
    assert!(
        html.contains("max-width: 950px")
            && html.contains(".btn-mob{ position:absolute")
            && html.contains("display:block"),
        "{path}: missing mobile header breakpoint rules"
    );
    assert!(
        html.contains("#day-night{ position: absolute; right: 80px; }"),
        "{path}: missing mobile day-night positioning"
    );
}

fn assert_live_listing_theme_state(html: &str, path: &str) {
    assert_theme_page_contract(html, path);
    assert!(
        html.contains("<html lang=\"en\" prefix=\"og: http://ogp.me/ns#\" >"),
        "{path}: listing shell should start in the live day theme"
    );
    assert!(
        html.contains("id=\"day-night\" title=\"Night mode\""),
        "{path}: missing live day-night control title"
    );
    assert!(
        html.contains("id=\"day-night-icon\" class=\"to-night\"")
            && html.contains("<use xlink:href=\"#moon-svg\" />"),
        "{path}: missing live moon icon state"
    );
    assert!(
        html.contains("screen_mode = 'd'"),
        "{path}: missing live listing boot global screen_mode = 'd'"
    );
}

fn assert_default_dark_theme_state(html: &str, path: &str) {
    assert_theme_page_contract(html, path);
    assert!(
        html.contains("<html lang=\"en\"") && html.contains("data-theme=\"dark\""),
        "{path}: missing default dark theme on <html>"
    );
    assert!(
        html.contains("id=\"day-night\" title=\"Day mode\""),
        "{path}: missing day-night control with Day mode title"
    );
    assert!(
        html.contains("id=\"day-night-icon\" class=\"to-day\"")
            && html.contains("<use xlink:href=\"#sun-svg\" />"),
        "{path}: missing default sun icon state"
    );
    assert!(
        html.contains("screen_mode = 'n'"),
        "{path}: missing default night-mode boot global screen_mode = 'n'"
    );
}

fn assert_main_min_js_theme_contract(js: &str) {
    assert!(
        js.contains("#day-night\").on(\"click\""),
        "main.min.js: missing #day-night click handler"
    );
    assert!(
        js.contains("document.documentElement.setAttribute(\"data-theme\",\"dark\")")
            && js.contains("document.documentElement.removeAttribute(\"data-theme\")"),
        "main.min.js: missing data-theme toggle on documentElement"
    );
    assert!(
        js.contains("set_cookie(\"sc_mod\",\"n\",3650)")
            && js.contains("set_cookie(\"sc_mod\",\"d\",3650)"),
        "main.min.js: missing sc_mod cookie writes"
    );
    assert!(
        js.contains("$(\"body\").addClass(\"black\")")
            && js.contains("$(\"body\").removeClass(\"black\")"),
        "main.min.js: missing body.black class toggles"
    );
    assert!(
        js.contains("$(\"#day-night\").attr(\"title\",\"Day mode\")")
            && js.contains("$(\"#day-night\").attr(\"title\",\"Night mode\")"),
        "main.min.js: missing day-night title updates"
    );
    assert!(
        js.contains("<use xlink:href=\"#sun-svg\" />")
            && js.contains("<use xlink:href=\"#moon-svg\" />"),
        "main.min.js: missing sun/moon icon swap markup"
    );
    assert!(
        js.contains("$(\"#day-night-icon\").addClass(\"to-day\")")
            && js.contains("$(\"#day-night-icon\").removeClass(\"to-night\")")
            && js.contains("$(\"#day-night-icon\").addClass(\"to-night\")")
            && js.contains("$(\"#day-night-icon\").removeClass(\"to-day\")"),
        "main.min.js: missing day-night icon class toggles"
    );
    assert!(
        !js.contains("localStorage"),
        "main.min.js: theme must not use localStorage"
    );
    assert!(
        !js.contains("get_cookie(\"sc_mod\")"),
        "main.min.js: sc_mod cookie is written on click but not read on load (document parity gap)"
    );
}

#[actix_web::test]
async fn home_page_includes_theme_toggle_markup_and_boot_state() {
    let html = fetch_html("/").await;
    assert_live_listing_theme_state(&html, "/");
    assert!(
        html.contains("src=\"/fox-tpl/js/main.min.js?v=11\""),
        "home should mirror the live main.min.js path"
    );

    let js = main_min_js_contract();
    assert_main_min_js_theme_contract(&js);
    let fox_tpl = fs::read_to_string("static/fox-tpl/js/main.min.js").expect("fox-tpl main.min.js");
    assert_main_min_js_theme_contract(&fox_tpl);
}

#[actix_web::test]
async fn categories_page_includes_theme_toggle_markup_and_boot_state() {
    let html = fetch_html("/categories").await;
    assert_live_listing_theme_state(&html, "/categories");
    assert!(
        html.contains("main.min.js"),
        "categories should reference main.min.js"
    );
}

#[actix_web::test]
async fn pornstars_page_includes_theme_toggle_markup_and_boot_state() {
    let html = fetch_html("/pornstars").await;
    assert_live_listing_theme_state(&html, "/pornstars");
}

#[actix_web::test]
async fn sample_video_page_includes_theme_toggle_markup_and_boot_state() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    let html = fetch_html(&path).await;
    assert_default_dark_theme_state(&html, &path);
    assert!(
        html.contains("isPLAYER = true"),
        "video page boot script should set isPLAYER"
    );
}
