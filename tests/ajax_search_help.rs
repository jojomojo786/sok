//! POST `/ajax/search_help` contract tests (sok-replica.5.1).

use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn post_search_help(payload: &'static str) -> actix_web::dev::ServiceResponse {
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
        .uri("/ajax/search_help")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(payload)
        .to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn post_search_help_returns_json_with_handler_marker() {
    let resp = post_search_help("text=milf").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("search_help")
    );
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/json; charset=utf-8")
    );
}

#[actix_web::test]
async fn post_search_help_echoes_text_and_group_item_shapes() {
    // "a" matches across fixtures and the live DB; production returns JSON
    // groups even for single-character queries.
    let resp = post_search_help("text=a").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");

    assert_eq!(v["search_text"].as_str(), Some("a"));
    assert!(v["pornstars"].is_array());
    assert!(v["channels"].is_array());
    assert!(v["videos"].is_array());

    let non_empty = !v["pornstars"].as_array().unwrap().is_empty()
        || !v["channels"].as_array().unwrap().is_empty()
        || !v["videos"].as_array().unwrap().is_empty();
    assert!(non_empty, "expected suggestions from DB or fixtures");

    for item in v["pornstars"].as_array().unwrap() {
        for key in ["url_pornstar", "name", "orig_name", "thumb", "count_videos"] {
            assert!(item.get(key).is_some(), "pornstar missing {key}");
        }
        assert!(item["url_pornstar"]
            .as_str()
            .unwrap()
            .starts_with("/pornstar/"));
        assert!(item["count_videos"].is_string());
    }
    for item in v["channels"].as_array().unwrap() {
        for key in ["url", "orig_name", "rus_name", "thumb", "count_videos"] {
            assert!(item.get(key).is_some(), "channel missing {key}");
        }
        assert!(item["url"].as_str().unwrap().starts_with("/channel/"));
    }
    for item in v["videos"].as_array().unwrap() {
        for key in ["url", "title", "thumb", "widethumb"] {
            assert!(item.get(key).is_some(), "video missing {key}");
        }
        let url = item["url"].as_str().unwrap();
        assert!(url.starts_with("/video/") && url.ends_with(".html"));
        assert!(item["widethumb"].is_string());
    }
}

#[actix_web::test]
async fn post_search_help_empty_query_returns_default_suggestions() {
    let resp = post_search_help("text=").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");

    assert_eq!(v["search_text"].as_str(), Some(""));
    let non_empty = !v["pornstars"].as_array().unwrap().is_empty()
        || !v["channels"].as_array().unwrap().is_empty()
        || !v["videos"].as_array().unwrap().is_empty();
    assert!(non_empty, "empty query should still return defaults");
}

#[actix_web::test]
async fn post_search_help_preserves_stale_echo_with_surrounding_whitespace() {
    // The stale guard in main.min.js compares the echoed `search_text` against
    // the raw input, so the handler must echo the submitted text verbatim.
    let resp = post_search_help("text=%20milf%20").await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");
    assert_eq!(v["search_text"].as_str(), Some(" milf "));
}
