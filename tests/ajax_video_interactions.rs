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
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["raiting"], 74);
    assert!(v["msg"].as_str().unwrap_or("").contains("Thanks"));
}

#[actix_web::test]
async fn post_add_vote_v3_rejects_unknown_video_id() {
    let resp = post_form("/ajax/add_vote_v3", "id_video=0&status=1".into()).await;
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["raiting"], 0);
    assert_eq!(v["msg"], "Unknown video.");
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
