//! Performance and payload-budget smoke checks.
//!
//! These tests document baseline response time and rendered HTML size for the
//! main page families and flag only extreme regressions. PornsOK pages are
//! intentionally heavy (large grids, inline SEO copy, emoji sprites), so the
//! budgets here are pragmatic ceilings meant to catch accidental explosions
//! (runaway loops, duplicated layout, unbounded fan-out) rather than to enforce
//! modern minimalism.
//!
//! See `docs/performance-payload-smoke.md` for the rationale and baselines.

use actix_web::{body::to_bytes, test, web, App};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;
use std::time::{Duration, Instant};

/// Upper bound for a single in-process render+serve cycle.
///
/// This is deliberately generous: the test harness shares the live DB pool and
/// CI machines are slow, so the goal is to flag pathological slowdowns (seconds,
/// not milliseconds), not to assert a tight latency SLO.
const MAX_RESPONSE_TIME: Duration = Duration::from_secs(10);

/// Hard ceiling on rendered HTML size for any single main page.
///
/// Real PornsOK pages land in the low hundreds of KB. 6 MiB leaves wide
/// headroom for legitimately heavy grids while still catching a render that has
/// exploded by an order of magnitude.
const MAX_HTML_BYTES: usize = 6 * 1024 * 1024;

/// Floor on rendered HTML size for a successful main page.
///
/// A full PornsOK layout (head, header, grid, footer, SEO copy) is always more
/// than a few KB. A 200 response under this size means the template collapsed to
/// a stub or error shell, which is itself a regression worth flagging.
const MIN_HTML_BYTES: usize = 4 * 1024;

struct PageMeasurement {
    status: actix_web::http::StatusCode,
    handler: Option<String>,
    bytes: usize,
    elapsed: Duration,
}

fn handler_name(resp: &actix_web::dev::ServiceResponse) -> Option<String> {
    resp.headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

macro_rules! measure {
    ($path:expr) => {{
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

        let req = test::TestRequest::get().uri($path).to_request();
        let start = Instant::now();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        let handler = handler_name(&resp);
        let body = to_bytes(resp.into_body())
            .now_or_never()
            .expect("body resolved synchronously")
            .expect("response body");
        let elapsed = start.elapsed();

        PageMeasurement {
            status,
            handler,
            bytes: body.len(),
            elapsed,
        }
    }};
}

/// Asserts the shared payload + latency budget for a page that must render a
/// full HTML document (HTTP 200).
fn assert_full_page_budget(path: &str, m: &PageMeasurement) {
    assert_eq!(
        m.status, 200,
        "{path} should render a full page (200), got {} via handler {:?}",
        m.status, m.handler
    );
    assert!(
        m.bytes >= MIN_HTML_BYTES,
        "{path} rendered only {} bytes (< {} floor); template likely collapsed to a stub",
        m.bytes,
        MIN_HTML_BYTES
    );
    assert!(
        m.bytes <= MAX_HTML_BYTES,
        "{path} rendered {} bytes (> {} ceiling); payload likely exploded",
        m.bytes,
        MAX_HTML_BYTES
    );
    assert!(
        m.elapsed <= MAX_RESPONSE_TIME,
        "{path} took {:?} (> {:?} budget); response time regressed sharply",
        m.elapsed,
        MAX_RESPONSE_TIME
    );

    // Document the observed baseline for this run. Visible with `--nocapture`.
    println!(
        "[perf-smoke] {path}: handler={:?} status={} bytes={} ({:.1} KiB) time={:?}",
        m.handler,
        m.status,
        m.bytes,
        m.bytes as f64 / 1024.0,
        m.elapsed
    );
}

#[actix_web::test]
async fn home_page_within_payload_and_time_budget() {
    let m = measure!("/");
    assert_eq!(m.handler.as_deref(), Some("index"));
    assert_full_page_budget("/", &m);
}

#[actix_web::test]
async fn categories_page_within_payload_and_time_budget() {
    let m = measure!("/categories");
    assert_eq!(m.handler.as_deref(), Some("categories"));
    assert_full_page_budget("/categories", &m);
}

#[actix_web::test]
async fn pornstars_page_within_payload_and_time_budget() {
    let m = measure!("/pornstars");
    assert_eq!(m.handler.as_deref(), Some("pornstars"));
    assert_full_page_budget("/pornstars", &m);
}

#[actix_web::test]
async fn channels_page_within_payload_and_time_budget() {
    let m = measure!("/channels");
    assert_eq!(m.handler.as_deref(), Some("channels"));
    assert_full_page_budget("/channels", &m);
}

#[actix_web::test]
async fn tags_page_within_payload_and_time_budget() {
    let m = measure!("/tags");
    assert_eq!(m.handler.as_deref(), Some("tags"));
    assert_full_page_budget("/tags", &m);
}

#[actix_web::test]
async fn category_slug_page_within_payload_and_time_budget() {
    let m = measure!("/milf");
    assert_eq!(m.handler.as_deref(), Some("category_slug"));
    assert_full_page_budget("/milf", &m);
}

#[actix_web::test]
async fn video_detail_page_within_payload_and_time_budget() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    let m = measure!(&path);
    assert_eq!(m.handler.as_deref(), Some("video_html"));
    assert_full_page_budget(&path, &m);
}

#[actix_web::test]
async fn legal_page_within_payload_and_time_budget() {
    let m = measure!("/page/privacy.html");
    assert_eq!(m.handler.as_deref(), Some("page_static"));
    assert_full_page_budget("/page/privacy.html", &m);
}

#[actix_web::test]
async fn critical_static_asset_serves_quickly_and_bounded() {
    // Release builds serve the JS bundle from disk via actix-files (a streaming
    // body), so this case reads the body asynchronously rather than through the
    // `measure!` macro. It guards against the static serving path regressing
    // badly in latency or growing absurdly large. The bundle is real production
    // JS, so the ceiling is generous.
    const MAX_STATIC_BYTES: usize = 8 * 1024 * 1024;

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
        .uri("/static/js/main.min.js")
        .to_request();
    let start = Instant::now();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    let elapsed = start.elapsed();
    let bytes = body.len();

    assert_eq!(status, 200, "main.min.js should serve 200");
    assert!(
        bytes > 0,
        "main.min.js should not be empty (got {bytes} bytes)"
    );
    assert!(
        bytes <= MAX_STATIC_BYTES,
        "main.min.js is {bytes} bytes (> {MAX_STATIC_BYTES} ceiling); static asset bloated unexpectedly"
    );
    assert!(
        elapsed <= MAX_RESPONSE_TIME,
        "serving main.min.js took {elapsed:?} (> {MAX_RESPONSE_TIME:?} budget)"
    );
    println!(
        "[perf-smoke] /static/js/main.min.js: status={} bytes={} ({:.1} KiB) time={:?}",
        status,
        bytes,
        bytes as f64 / 1024.0,
        elapsed
    );
}
