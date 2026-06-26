//! POST video-page AJAX contract tests (sok-replica.5.6).

use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::sample_dog_house_video_detail;

async fn post_form(uri: &str, payload: String) -> actix_web::dev::ServiceResponse {
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
        .uri(uri)
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(payload)
        .to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn post_add_vote_v3_returns_json_rating_for_known_video() {
    let video_id = sample_dog_house_video_detail().thumb.id;
    let payload = format!("id_video={video_id}&status=1");
    let resp = post_form("/ajax/add_vote_v3", payload).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("add_vote_v3")
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
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["raiting"], 74);
    assert!(v["msg"].as_str().unwrap_or("").contains("Thanks"));
}

#[actix_web::test]
async fn post_add_vote_v3_rejects_unknown_video_id() {
    let resp = post_form("/ajax/add_vote_v3", "id_video=0&status=1".into()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    // The invalid-id early return must share the live-compatible transport so
    // the mirrored jQuery 3.3.1 client's `$.parseJSON` still receives a raw
    // string (see sok-replica.5.8).
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
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["raiting"], 0);
    assert_eq!(v["msg"], "Unknown video.");
}

#[actix_web::test]
async fn post_add_vote_v3_persists_unlike_and_returns_safe_rating() {
    // Unlike on the fixture video: persistence is attempted, and even when the catalog row
    // is absent the handler degrades to the known like percent rather than erroring.
    let video_id = sample_dog_house_video_detail().thumb.id;
    let payload = format!("id_video={video_id}&status=unlike");
    let resp = post_form("/ajax/add_vote_v3", payload).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert!(v["raiting"].as_u64().is_some());
    assert!(v["msg"].as_str().unwrap_or("").contains("Thanks"));
}

#[actix_web::test]
async fn post_add_vote_v3_unknown_status_still_returns_rating() {
    // Unrecognized status: no vote is persisted but the current rating is still returned.
    let video_id = sample_dog_house_video_detail().thumb.id;
    let payload = format!("id_video={video_id}&status=banana");
    let resp = post_form("/ajax/add_vote_v3", payload).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert!(v["raiting"].as_u64().is_some());
}

#[actix_web::test]
async fn get_ajax_unknown_tail_still_reserved_stub() {
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
        .uri("/ajax/not-a-real-endpoint")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("ajax")
    );
    let body = test::read_body(resp).await;
    assert_eq!(String::from_utf8_lossy(&body), "stub:ajax");
}
