//! POST `/ajax/search_{type}` contract tests (sok-replica.5.3).

use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn post_entity_search(path: &str, payload: String) -> actix_web::dev::ServiceResponse {
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
        .set_payload(payload)
        .to_request();
    test::call_service(&app, req).await
}

fn assert_card_item(item: &Value, profile_prefix: &str) {
    for key in ["url", "thumb", "orig_name", "count_videos"] {
        assert!(item.get(key).is_some(), "missing {key}");
    }
    assert!(item["url"].as_str().unwrap().starts_with(profile_prefix));
    assert!(item["thumb"].as_str().unwrap().starts_with("http"));
    assert!(item["count_videos"].is_number());
}

#[actix_web::test]
async fn post_search_pstars_returns_json_with_handler_marker() {
    let resp = post_entity_search("/ajax/search_pstars", "text=ang".to_string()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("search_pornstars")
    );
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/json; charset=utf-8")
    );
}

#[actix_web::test]
async fn post_search_channels_returns_json_with_handler_marker() {
    let resp = post_entity_search("/ajax/search_channels", "text=bra".to_string()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("search_channels")
    );
}

#[actix_web::test]
async fn post_search_pstars_echoes_search_text_and_card_shape() {
    let query = "ang";
    let resp = post_entity_search("/ajax/search_pstars", format!("text={query}")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");

    assert_eq!(v["search_text"].as_str(), Some(query));
    let items = v["items"].as_array().expect("items array");
    assert!(!items.is_empty(), "expected pornstar matches");
    for item in items {
        assert_card_item(item, "/pornstar/");
    }
}

#[actix_web::test]
async fn post_search_channels_echoes_search_text_and_card_shape() {
    let query = "bra";
    let resp = post_entity_search("/ajax/search_channels", format!("text={query}")).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");

    assert_eq!(v["search_text"].as_str(), Some(query));
    let items = v["items"].as_array().expect("items array");
    assert!(!items.is_empty(), "expected channel matches");
    for item in items {
        assert_card_item(item, "/channel/");
    }
}

#[actix_web::test]
async fn post_search_pstars_short_query_returns_empty_items() {
    let resp = post_entity_search("/ajax/search_pstars", "text=a".to_string()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["search_text"].as_str(), Some("a"));
    assert_eq!(v["items"].as_array().map(|a| a.len()), Some(0));
}

#[actix_web::test]
async fn unsupported_entity_search_type_is_not_routed() {
    let resp = post_entity_search("/ajax/search_videos", "text=milf".to_string()).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("ajax")
    );
}
