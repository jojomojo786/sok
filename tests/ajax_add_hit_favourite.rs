use actix_web::{http::StatusCode, test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn post_add_hit_favourite(payload: Option<String>) -> actix_web::dev::ServiceResponse {
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
    let mut req = test::TestRequest::post().uri("/ajax/add_hit/favourite");
    if let Some(body) = payload {
        req = req
            .insert_header(("Content-Type", "application/x-www-form-urlencoded"))
            .set_payload(body);
    }
    let req = req.to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn post_add_hit_favourite_empty_body_succeeds() {
    let resp = post_add_hit_favourite(None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("add_hit_favourite")
    );
    let body = test::read_body(resp).await;
    assert!(body.is_empty(), "expected empty analytics response body");
}

#[actix_web::test]
async fn post_add_hit_favourite_ignores_payload_and_still_succeeds() {
    let resp = post_add_hit_favourite(Some("unexpected=1&also=2".into())).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("add_hit_favourite")
    );
    let body = test::read_body(resp).await;
    assert!(body.is_empty());
}
