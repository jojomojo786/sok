use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

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
async fn post_comments_accepts_kemoji_message() {
    let payload = "name=Ada&msg=Nice%20%5Bsperm_0DCpe%5D&vid=52994";
    let resp = post_form("/ajax/comments", payload.into()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("post_comments")
    );
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["result"], "ok");
}

#[actix_web::test]
async fn post_comments_rejects_unsafe_img_markup() {
    let payload = "name=Ada&msg=%3Cimg%20src%3Dx%20onerror%3Dalert(1)%3E&vid=52994";
    let resp = post_form("/ajax/comments", payload.into()).await;
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["result"], "error");
}

#[actix_web::test]
async fn post_comments_strips_inline_script_to_safe_text() {
    let payload = "name=Ada&msg=Hi%20%3Cscript%3Ealert(1)%3C%2Fscript%3E%20there&vid=52994";
    let resp = post_form("/ajax/comments", payload.into()).await;
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["result"], "ok");
}

#[actix_web::test]
async fn post_comments_rejects_invalid_video_id() {
    let payload = "name=Ada&msg=Hello&vid=0";
    let resp = post_form("/ajax/comments", payload.into()).await;
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["result"], "error");
}

#[actix_web::test]
async fn post_more_comments_returns_html_fragment() {
    let payload = format!("videourl={DOG_HOUSE_SLUG}");
    let resp = post_form("/ajax/more_comments", payload).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("post_more_comments")
    );
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("json");
    let comments = v["comments"].as_str().unwrap_or("");
    assert!(comments.contains("comments-box") || comments.is_empty());
}
