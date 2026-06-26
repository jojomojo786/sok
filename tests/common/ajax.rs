use actix_web::{test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

pub async fn post_form(path: &str, payload: &str) -> actix_web::dev::ServiceResponse {
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
        .uri(path)
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(payload.to_string())
        .to_request();
    test::call_service(&app, req).await
}

pub async fn post_optional_body(
    path: &str,
    payload: Option<&str>,
) -> actix_web::dev::ServiceResponse {
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
    let mut req = test::TestRequest::post().uri(path);
    if let Some(body) = payload {
        req = req
            .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
            .set_payload(body.to_string());
    }
    let req = req.to_request();
    test::call_service(&app, req).await
}

pub fn handler_marker(resp: &actix_web::dev::ServiceResponse) -> Option<&str> {
    resp.headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
}

pub async fn json_body(resp: actix_web::dev::ServiceResponse) -> Value {
    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).expect("valid json")
}

pub fn assert_object_has_only_keys(obj: &Value, keys: &[&str]) {
    let map = obj.as_object().expect("json object");
    for key in keys {
        assert!(map.contains_key(*key), "missing key {key}");
    }
    assert_eq!(
        map.len(),
        keys.len(),
        "unexpected extra keys: {:?}",
        map.keys().collect::<Vec<_>>()
    );
}
