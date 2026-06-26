use std::fs;
use std::path::PathBuf;

use actix_web::{http::StatusCode, test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers;

async fn fetch_home_html() -> String {
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
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    String::from_utf8_lossy(&body).into_owned()
}

fn main_min_js() -> String {
    for path in [
        PathBuf::from("static/js/main.min.js"),
        PathBuf::from("static/fox-tpl/js/main.min.js"),
    ] {
        if path.is_file() {
            return fs::read_to_string(&path).expect("read main.min.js");
        }
    }
    panic!("main.min.js not found");
}

fn assert_home_nav_markup(html: &str) {
    assert!(
        html.contains("class=\"header-in center clearfix\""),
        "missing header shell"
    );
    assert!(
        html.contains("class=\"header-menu\""),
        "missing desktop header menu"
    );
    assert!(
        html.contains("class=\"nav-show\""),
        "missing mega menu nav-show items"
    );
    assert!(
        html.contains("id=\"menu-top-list\" class=\"submenu-container\""),
        "missing submenu container"
    );
    assert!(
        html.contains("#menu-top-list.hovered"),
        "missing hovered mega menu CSS hook"
    );
    assert!(
        html.contains("class=\"btn-search activate\""),
        "missing search toggle"
    );
    assert!(
        html.contains("id=\"main-search\""),
        "missing main search input"
    );
    assert!(
        html.contains("class=\"btn-mob\""),
        "missing mobile menu button"
    );
    assert!(
        html.contains("max-width: 949px")
            && html.contains("footer-logo.svg")
            && html.contains("spacer.gif"),
        "missing footer picture breakpoint sources"
    );
    assert!(
        html.contains("header-menu,.nav, i.d-menu{ display:none }")
            && html.contains(".btn-mob{ position:absolute; right:18px"),
        "missing mobile header breakpoint rules"
    );
    assert!(
        html.contains("#day-night{ position: absolute; right: 80px; }"),
        "missing mobile day-night positioning"
    );
    assert!(
        html.contains(".footer-link {display: none;}"),
        "missing mobile footer-link hide rule"
    );
    assert!(
        html.contains(".wrap{ min-width:320px; z-index:9999; padding-top:70px }"),
        "missing 70px wrap offset"
    );
}

fn assert_main_min_js_nav_contract(js: &str) {
    assert!(
        js.contains("$(\".nav-show\").on(\"mouseover\""),
        "missing mega menu mouseover handler"
    );
    assert!(
        js.contains("e.addClass(\"hovered\")"),
        "missing submenu hovered class add"
    );
    assert!(
        js.contains("removeClass(\"hovered\")"),
        "missing submenu hovered class remove"
    );
    assert!(
        js.contains("$(\".btn-search.activate\").mouseup"),
        "missing search expand handler"
    );
    assert!(
        js.contains("$(\"body\").append('<div class=\"close-overlay\" id=\"close-overlay\"></div><ul class=\"side-panel\" id=\"side-panel\"></ul>')"),
        "missing side panel injection"
    );
    assert!(
        js.contains("$(\".btn-mob\").click"),
        "missing mobile menu open handler"
    );
    assert!(
        js.contains("$(\".close-overlay\").click"),
        "missing overlay close handler"
    );
    assert!(
        js.contains("url:\"/ajax/search_help\""),
        "missing search_help AJAX endpoint"
    );
}

#[actix_web::test]
async fn home_page_includes_responsive_navigation_markup() {
    let html = fetch_home_html().await;
    assert_home_nav_markup(&html);
}

#[actix_web::test]
async fn main_min_js_includes_responsive_navigation_contract() {
    let js = main_min_js();
    assert_main_min_js_nav_contract(&js);
}
