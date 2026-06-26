use actix_web::{test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};

fn handler_name(resp: &actix_web::dev::ServiceResponse) -> Option<String> {
    resp.headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

macro_rules! call {
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
        test::call_service(&app, req).await
    }};
}

#[actix_web::test]
async fn home_renders_dynamic_grid_meta_filters_and_seo() {
    let resp = call!("/");
    assert_eq!(resp.status(), 200);
    assert_eq!(handler_name(&resp).as_deref(), Some("index"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);

    // Title / canonical / H1.
    assert!(
        html.contains("Free Porn Videos &amp; Hot 🌶️ Sex Movies | PornsOK.com</title>")
            || html.contains("Free Porn Videos & Hot 🌶️ Sex Movies | PornsOK.com</title>"),
        "missing home title"
    );
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/""#),
        "missing canonical"
    );
    assert!(
        html.contains("<h1>Top Trending Free Porn Videos</h1>"),
        "missing H1"
    );

    // Non-empty dynamic grid with real video links and production classes.
    assert!(
        html.contains(r#"<div class="thumbs-floats">"#),
        "missing grid wrapper"
    );
    assert!(html.contains("thumb vid"), "missing thumb class");
    assert!(html.contains("thumb-cover"), "missing thumb-cover");
    assert!(html.contains("/video/"), "missing video links");
    // Dynamic loop must emit at least one rendered thumb (DB or fixture fallback).
    assert!(
        html.matches("<!-- / thumb -->").count() >= 1,
        "dynamic home grid is empty"
    );

    // Filter links.
    assert!(html.contains(r#"href="/?hd=1""#), "missing HD filter link");
    assert!(
        html.contains(r#"href="/?sort=mv""#),
        "missing most-viewed sort link"
    );
    assert!(
        html.contains(r#"href="/?sort=mc""#),
        "missing most-commented sort link"
    );

    // Pagination wrapper renders when there is at least one page of data.
    assert!(
        html.contains(r#"class="page_nav""#),
        "missing pagination wrapper"
    );

    // SEO copy (intro + footer).
    assert!(
        html.contains("Horny guys looking for the best free porn video"),
        "missing SEO intro"
    );
    assert!(
        html.contains("Masturbating has never been this fun!"),
        "missing SEO footer"
    );
}

#[actix_web::test]
async fn home_page_two_uses_dynamic_handler_not_stub() {
    let resp = call!("/2");
    // With limited fixture data page 2 is out of range and returns the 404 page;
    // with a populated DB it renders the dynamic home template. Either way the
    // old `stub:home_page_num` response must be gone.
    let handler = handler_name(&resp);
    assert!(
        matches!(
            handler.as_deref(),
            Some("home_page_num") | Some("not_found")
        ),
        "unexpected handler {handler:?} for /2"
    );
    let status = resp.status();
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(!html.starts_with("stub:"), "page 2 should not be a stub");
    if status == 200 {
        assert!(
            html.contains("<h1>Top Trending Free Porn Videos</h1>"),
            "missing H1 on page 2"
        );
        assert!(
            html.contains(r#"canonical" href="https://pornsok.com/2""#),
            "missing page-2 canonical"
        );
        assert!(html.contains(r#"rel="prev""#), "missing rel=prev on page 2");
        assert!(
            html.contains(r#"<div class="thumbs-floats">"#),
            "missing grid wrapper"
        );
    }
}

#[actix_web::test]
async fn home_sort_mv_marks_most_viewed_selected() {
    let resp = call!("/?sort=mv");
    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);

    // Sort links carry the active sort through their hrefs.
    assert!(html.contains(r#"href="/?sort=mv""#), "missing mv sort href");
    // Selected marker present in the select_sort block.
    assert!(html.contains(" selected "), "missing selected sort marker");
    assert!(
        html.contains(r#"<div class="thumbs-floats">"#),
        "missing grid wrapper"
    );
}

#[actix_web::test]
async fn home_hd_filter_marks_hd_active() {
    let resp = call!("/?hd=1");
    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    // HD anchor should be active when hd=1.
    assert!(
        html.contains(r#"<a class=" active " href="/?hd=1" rel="nofollow">HD</a>"#),
        "HD filter not marked active"
    );
}

#[actix_web::test]
async fn home_sort_mv_canonical_points_to_base_home() {
    let resp = call!("/?sort=mv");
    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/""#),
        "sort=mv should canonicalize to base home"
    );
    assert!(
        !html.contains(r#"canonical" href="https://pornsok.com/?sort=mv""#),
        "canonical must not echo sort query"
    );
}

#[actix_web::test]
async fn home_hd_filter_canonical_points_to_base_home() {
    let resp = call!("/?hd=1");
    assert_eq!(resp.status(), 200);
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"canonical" href="https://pornsok.com/""#),
        "hd=1 should canonicalize to base home"
    );
}
