//! POST `/ajax/update_*` homepage widget contract tests (sok-replica.5.4).

use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn post_empty(path: &str) -> actix_web::dev::ServiceResponse {
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
        .set_payload("")
        .to_request();
    test::call_service(&app, req).await
}

async fn post_form(path: &str, payload: &str) -> actix_web::dev::ServiceResponse {
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

#[actix_web::test]
async fn post_update_pornstars_returns_cat_thumb_html() {
    let resp = post_empty("/ajax/update_pornstars").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("update_pornstars")
    );
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains(r#"class="thumb cat""#));
    assert!(body.contains("count-videos"));
    assert!(body.contains("/pornstar/"));
}

#[actix_web::test]
async fn post_update_channels_returns_cat_thumb_html() {
    let resp = post_empty("/ajax/update_channels").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("update_channels")
    );
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains(r#"class="thumb cat""#));
    assert!(body.contains("/channel/"));
}

#[actix_web::test]
async fn post_update_watching_now_returns_vid_thumb_html() {
    let resp = post_form("/ajax/update_watching_now", "order_by=week_views").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("update_watching_now")
    );
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains(r#"class="thumb vid""#));
    assert!(body.contains("data-video="));
    assert!(body.contains("fa-eye"));
}

#[actix_web::test]
async fn post_update_newest_videos_returns_vid_thumb_html() {
    let resp = post_form("/ajax/update_newest_videos", "video_id=1&offset=0&count=12").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("update_newest_videos")
    );
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains(r#"class="thumb vid""#));
    assert!(body.contains("/video/"));
}

#[actix_web::test]
async fn post_update_tags_returns_json_shape() {
    let resp = post_empty("/ajax/update_tags").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("update_tags")
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
    assert!(v.get("html").and_then(|x| x.as_str()).is_some());
    assert!(v.get("preload_before").and_then(|x| x.as_str()).is_some());
    assert!(v.get("preload_after").and_then(|x| x.as_str()).is_some());
    assert!(v.get("preload_array").and_then(|x| x.as_array()).is_some());
    let html = v["html"].as_str().unwrap();
    assert!(html.contains("fa-tag"));
}
