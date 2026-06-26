use actix_web::{body::to_bytes, test, web, App, ResponseError};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::errors::{AppError, PUBLIC_INTERNAL_ERROR_MESSAGE};
use sok::handlers::{self, HANDLER_MARKER};

async fn get_response(path: &str) -> actix_web::dev::ServiceResponse {
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
    let req = test::TestRequest::get().uri(path).to_request();
    test::call_service(&app, req).await
}

#[actix_web::test]
async fn unknown_slug_returns_styled_404_with_category_handler() {
    let slug = format!(
        "no-such-taxonomy-slug-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let path = format!("/{slug}");
    let resp = get_response(&path).await;
    assert_eq!(resp.status(), 404);
    assert_eq!(
        resp.headers()
            .get(HANDLER_MARKER)
            .and_then(|v| v.to_str().ok()),
        Some("category_slug")
    );
    let body = to_bytes(resp.into_body()).now_or_never().unwrap();
    let html = String::from_utf8(body.expect("response body").to_vec()).unwrap();
    assert!(html.contains("Page not found"));
    assert!(html.contains("footer"));
    assert!(!html.contains("stub:category"));
}

#[actix_web::test]
async fn app_error_db_response_hides_secrets() {
    let err = AppError::Db(sqlx::Error::Configuration(
        "mysql://leak_user:leak_pass@host/db".into(),
    ));
    let resp = err.error_response();
    assert_eq!(resp.status(), 500);
    let body = to_bytes(resp.into_body()).now_or_never().unwrap();
    let html = String::from_utf8(body.expect("response body").to_vec()).unwrap();
    assert!(!html.contains("leak_pass"));
    assert!(!html.contains("leak_user"));
    assert!(html.contains(PUBLIC_INTERNAL_ERROR_MESSAGE));
}
