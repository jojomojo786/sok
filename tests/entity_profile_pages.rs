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
async fn angela_white_profile_renders_meta_grid_and_header() {
    let resp = call!("/pornstar/angela-white");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("pornstar_profile"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Angela White Free Porn Videos and Scenes | PornsOK.com"));
    assert!(html.contains("<h1>Angela White - Newest Videos</h1>"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/pornstar/angela-white""#));
    assert!(html.contains("thumbs-floats"));
    assert!(html.contains("/video/"));
    assert!(html.contains("id=\"head-banner\""));
    assert!(html.contains("select_sort"));
}

#[actix_web::test]
async fn brazzers_channel_profile_renders_meta_grid_and_header() {
    let resp = call!("/channel/brazzers");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("channel_profile"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("Brazzers Porn Channel - Free Sex Videos | PornsOK.com"));
    assert!(html.contains("<h1>Brazzers - Latest Videos</h1>"));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/channel/brazzers""#));
    assert!(html.contains("thumbs-floats"));
    assert!(html.contains("/video/"));
    assert!(html.contains("id=\"head-banner\""));
}

#[actix_web::test]
async fn unknown_pornstar_profile_returns_404() {
    let resp = call!("/pornstar/this-profile-does-not-exist-xyz");
    assert_eq!(resp.status(), 404);
    assert_eq!(handler_name(&resp).as_deref(), Some("pornstar_profile"));
}
