//! POST `/ajax/search_cats_tags_queries` contract tests (sok-replica.5.2).

use actix_web::{http::StatusCode, test, web, App};
use serde_json::Value;
use sok::config::Config;
use sok::configure_static;
use sok::db;

use sok::handlers::{self, HANDLER_MARKER};

#[actix_web::test]
async fn post_search_cats_tags_returns_json_with_handler_marker() {
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
        .uri("/ajax/search_cats_tags_queries")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload("text=milf")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("search_cats_tags_queries")
    );
    // Live pornsok.com serves this JSON-bodied autocomplete response as
    // `text/html` with `nosniff` so the mirrored jQuery 3.3.1 client's
    // `$.parseJSON(responseText)` receives a raw string (see sok-replica.5.8).
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string()),
        Some("text/html; charset=utf-8".to_string())
    );
    assert_eq!(
        resp.headers()
            .get("x-content-type-options")
            .and_then(|v| v.to_str().ok()),
        Some("nosniff")
    );

    // Body must still parse as JSON for the contract consumers.
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");
    assert_eq!(v["search_text"].as_str(), Some("milf"));
}

#[actix_web::test]
async fn post_search_cats_tags_milf_matches_live_sample_body() {
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
        .uri("/ajax/search_cats_tags_queries")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload("text=milf")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    assert_eq!(
        body.as_ref(),
        include_bytes!("../docs/raw/search_cats_tags_queries.body")
    );
}

#[actix_web::test]
async fn post_search_cats_tags_echoes_search_text_and_item_shape() {
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
    let query = "big";
    let req = test::TestRequest::post()
        .uri("/ajax/search_cats_tags_queries")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload(format!("text={query}"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).expect("valid json");

    assert_eq!(v["search_text"].as_str(), Some(query));
    let items = v["items"].as_array().expect("items array");
    assert!(
        !items.is_empty(),
        "expected at least one match for fixture/DB"
    );

    for item in items {
        assert!(item.get("id").and_then(|x| x.as_str()).is_some());
        assert!(item.get("name").and_then(|x| x.as_str()).is_some());
        assert!(item.get("url").and_then(|x| x.as_str()).is_some());
        let url = item["url"].as_str().unwrap();
        assert!(url.starts_with('/'));
    }
}

#[actix_web::test]
async fn post_search_cats_tags_short_query_returns_empty_items() {
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
        .uri("/ajax/search_cats_tags_queries")
        .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
        .set_payload("text=a")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body = test::read_body(resp).await;
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["search_text"].as_str(), Some("a"));
    assert_eq!(v["items"].as_array().map(|a| a.len()), Some(0));
}
