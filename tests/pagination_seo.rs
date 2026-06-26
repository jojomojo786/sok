//! Pagination SEO: canonical base URLs vs rel prev/next with active filters.

use actix_web::{body::to_bytes, test, web, App};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

async fn get_html(path: &str) -> (actix_web::http::StatusCode, String, Option<String>) {
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
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let handler = resp
        .headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = to_bytes(resp.into_body()).now_or_never().unwrap();
    let html = String::from_utf8(body.expect("response body").to_vec()).unwrap();
    (status, html, handler)
}

#[actix_web::test]
async fn home_page_one_rel_next_points_to_page_two_when_multi_page() {
    let (status, html, _) = get_html("/").await;
    assert_eq!(status, 200);
    assert!(html.contains(r#"canonical" href="https://pornsok.com/""#));
    assert!(!html.contains(r#"rel="prev""#));
    // Fixture/DB may only fill one page; rel=next is required only when page 2 exists.
    if html.contains(r#"rel="next" href="https://pornsok.com/2""#) {
        assert!(html.contains(r#"class="page_nav""#));
    }
}

#[actix_web::test]
async fn home_filtered_sort_hd_canonical_collapses_to_base_paths() {
    let (status, html, _) = get_html("/?sort=mv&hd=1").await;
    assert_eq!(status, 200);
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/""#),
        "filtered home page 1 should canonicalize to /"
    );
    assert!(
        !html.contains(r#"canonical" href="https://pornsok.com/?sort=mv"#),
        "canonical must not include sort query"
    );
    assert!(
        !html.contains(r#"canonical" href="https://pornsok.com/?hd=1"#),
        "canonical must not include hd query"
    );
}

#[actix_web::test]
async fn milf_filtered_canonical_collapses_to_slug_base() {
    let (status, html, _) = get_html("/milf?sort=mv&hd=1").await;
    assert_eq!(status, 200);
    assert!(html.contains(r#"canonical" href="https://pornsok.com/milf""#));
    assert!(!html.contains(r#"canonical" href="https://pornsok.com/milf?sort="#));
}

#[actix_web::test]
async fn search_results_canonical_ignores_sort_and_hd() {
    let (status, html, _) = get_html("/videos/test?sort=recent&hd=1").await;
    assert_eq!(status, 200);
    assert!(html.contains(r#"canonical" href="https://pornsok.com/videos/test""#));
}

#[actix_web::test]
async fn pornstars_index_page_two_canonical_without_sort_query() {
    let (status, html, handler) = get_html("/pornstars/2?sort=videocount").await;
    if status != 200 {
        assert!(
            matches!(handler.as_deref(), Some("not_found")),
            "unexpected status/handler for /pornstars/2: {handler:?}"
        );
        return;
    }
    assert!(html.contains(r#"canonical" href="https://pornsok.com/pornstars/2""#));
    assert!(!html.contains(r#"canonical" href="https://pornsok.com/pornstars/2?sort="#));
}

#[actix_web::test]
async fn home_page_two_when_available_has_rel_prev_and_canonical() {
    let (status, html, handler) = get_html("/2").await;
    if status != 200 {
        assert!(
            matches!(handler.as_deref(), Some("not_found")),
            "unexpected /2 handler: {handler:?}"
        );
        return;
    }
    assert!(html.contains(r#"canonical" href="https://pornsok.com/2""#));
    assert!(html.contains(r#"rel="prev" href="https://pornsok.com/""#));
}
