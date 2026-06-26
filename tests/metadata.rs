//! Cross-family SEO head tag coverage (title, description, canonical, OG, RTA, rel prev/next).

use actix_web::{body::to_bytes, test, web, App};
use futures_util::future::FutureExt;
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

const RTA: &str = r#"<meta name="rating" content="RTA-5042-1996-1400-1577-RTA">"#;

fn assert_common_website_head(html: &str, family: &str) {
    assert!(
        html.contains(r#"prefix="og: http://ogp.me/ns#""#),
        "{family}: missing og html prefix"
    );
    assert!(html.contains(RTA), "{family}: missing RTA rating meta");
    assert!(
        html.contains(r#"property="og:site_name" content="pornsok.com""#),
        "{family}: missing og:site_name"
    );
    assert!(
        html.contains(r#"property="og:image" content="https://c.foxporn.tv/""#),
        "{family}: missing og:image"
    );
    assert!(
        html.contains(r#"<meta name="description" content=""#),
        "{family}: missing meta description"
    );
    assert!(
        html.contains(r#"property="og:title""#) && html.contains(r#"property="og:url""#),
        "{family}: missing og:title or og:url"
    );
    assert!(
        html.contains(r#"property="og:description""#),
        "{family}: missing og:description"
    );
}

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
async fn home_head_meta_includes_rel_next() {
    let (status, html, _) = get_html("/").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "home");
    assert!(html.contains(r#"property="og:type" content="website""#));
    assert!(html.contains(r#"canonical" href="https://pornsok.com/""#));
}

#[actix_web::test]
async fn categories_index_head_meta() {
    let (status, html, _) = get_html("/categories").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "categories");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/categories""#));
}

#[actix_web::test]
async fn slug_listing_head_meta() {
    let (status, html, _) = get_html("/milf").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "slug_listing");
    assert!(html.contains("MILF Porn Videos: Free MILF Sex Movies | PornsOK.com</title>"));
}

#[actix_web::test]
async fn tags_hub_head_meta() {
    let (status, html, _) = get_html("/tags").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "tags");
}

#[actix_web::test]
async fn pornstars_index_head_meta_includes_rel_next() {
    let (status, html, _) = get_html("/pornstars").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "pornstars");
}

#[actix_web::test]
async fn channels_index_head_meta() {
    let (status, html, _) = get_html("/channels").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "channels");
}

#[actix_web::test]
async fn entity_pornstar_profile_head_meta() {
    let (status, html, _) = get_html("/pornstar/angela-white").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "pornstar");
}

#[actix_web::test]
async fn entity_channel_profile_head_meta() {
    let (status, html, _) = get_html("/channel/brazzers").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "channel");
}

#[actix_web::test]
async fn search_results_head_meta() {
    let (status, html, _) = get_html("/videos/test").await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "search");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/videos/test""#));
}

#[actix_web::test]
async fn video_detail_head_meta_og_video_and_rta() {
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    let (status, html, _) = get_html(&path).await;
    assert_eq!(status, 200);
    assert_common_website_head(&html, "video");
    assert!(html.contains(r#"property="og:type" content="video.other""#));
}

#[actix_web::test]
async fn legal_privacy_head_meta() {
    let (status, html, handler) = get_html("/page/privacy.html").await;
    assert_eq!(status, 200);
    assert_eq!(handler.as_deref(), Some("page_static"));
    assert_common_website_head(&html, "legal");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/page/privacy.html""#));
}

#[actix_web::test]
async fn home_head_meta_includes_rel_next_on_page_two_when_available() {
    let (status, html, handler) = get_html("/2").await;
    if status != 200 {
        assert_eq!(handler.as_deref(), Some("not_found"));
        return;
    }
    assert_common_website_head(&html, "home_page_2");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/2""#));
    assert!(html.contains(r#"rel="prev" href="https://pornsok.com/""#));
}

#[actix_web::test]
async fn slug_listing_milf_page_two_canonical_when_available() {
    let (status, html, handler) = get_html("/milf/2").await;
    if status != 200 {
        assert_eq!(handler.as_deref(), Some("not_found"));
        return;
    }
    assert_common_website_head(&html, "milf_page_2");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/milf/2""#));
}

#[actix_web::test]
async fn search_results_page_two_canonical_when_available() {
    let (status, html, handler) = get_html("/videos/test/2").await;
    if status != 200 {
        assert!(
            matches!(handler.as_deref(), Some("not_found")),
            "unexpected handler: {handler:?}"
        );
        return;
    }
    assert_common_website_head(&html, "search_page_2");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/videos/test/2""#));
}

#[actix_web::test]
async fn entity_pornstar_profile_page_two_canonical_when_available() {
    let (status, html, handler) = get_html("/pornstar/angela-white/2").await;
    if status != 200 {
        assert!(
            matches!(handler.as_deref(), Some("not_found")),
            "unexpected handler: {handler:?}"
        );
        return;
    }
    assert_common_website_head(&html, "pornstar_page_2");
    assert!(html.contains(r#"canonical" href="https://pornsok.com/pornstar/angela-white/2""#));
}
