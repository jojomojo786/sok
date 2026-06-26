use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

async fn post_more_videos(payload: String) -> actix_web::dev::ServiceResponse {
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
        .uri("/ajax/more_videos_3")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(payload)
        .to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn post_more_videos_3_returns_json_batches() {
    let payload = format!("videourl={DOG_HOUSE_SLUG}");
    let resp = post_more_videos(payload).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("more_videos_3")
    );
    // Live pornsok.com serves this JSON body as `text/html` + `nosniff` so the
    // mirrored jQuery 3.3.1 client's `$.parseJSON` receives a raw string (see
    // sok-replica.5.8).
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        resp.headers()
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
    let first_batch = &v[0];
    assert!(first_batch.is_array());
    let item = &first_batch[0];
    for key in [
        "url",
        "title",
        "thumb",
        "preview_mini",
        "duration",
        "views",
        "str_views",
        "rate",
        "widethumb",
    ] {
        assert!(item.get(key).is_some(), "missing {key}");
    }
    assert!(item["url"].as_str().unwrap().ends_with(".html"));
}

#[actix_web::test]
async fn post_add_hit_more_videos_does_not_error() {
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
        .uri("/ajax/add_hit/more_videos")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("add_hit_more_videos")
    );
}
