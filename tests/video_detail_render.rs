use actix_web::{test, web, App};
use sok::config::Config;
use sok::configure_static;
use sok::db;
use sok::handlers::{self, HANDLER_MARKER};
use sok::models::video::fixtures::DOG_HOUSE_SLUG;

#[actix_web::test]
async fn video_detail_dog_house_renders_fixture_shell() {
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
    let path = format!("/video/{DOG_HOUSE_SLUG}.html");
    let req = test::TestRequest::get().uri(&path).to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let handler = resp
        .headers()
        .get(HANDLER_MARKER)
        .and_then(|v| v.to_str().ok());
    assert_eq!(handler, Some("video_html"));
    let body = test::read_body(resp).await;
    let html = String::from_utf8_lossy(&body);
    assert!(html.contains("player_container2"));
    assert!(html.contains("v-meta"));
    assert!(html.contains("v-tags"));
    assert!(html.contains("related-items"));
    assert!(html.contains(r#"id="more_video""#));
    assert!(html.contains(r#"id="leave_comment""#));
    assert!(html.contains(r#"id="comments-container""#));
    assert!(html.contains(r#"id="myEmoji""#));
    assert!(html.contains("comments-box"));
    assert!(html.contains("ke-sperm_0DCpe") || html.contains("[sperm_0DCpe]"));
    assert!(html.contains("isPLAYER = true"));
    assert!(html.contains("Madi Collins"));
    assert!(html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D"));
    assert!(html.contains("http://schema.org/VideoObject"));
    assert!(html.contains(r#"itemprop="description""#));
    assert!(html.contains(r#"itemprop="datePublished" content="2026-03-02""#));
    assert!(html.contains(r#"itemprop="duration" content="PT9M59S""#));
    assert!(html.contains("interactionStatistic"));
    assert!(html.contains("WatchAction"));
    assert!(html.contains("CommentAction"));
    assert!(html.contains(r#"userInteractionCount" content="15077""#));
    assert!(html.contains(r#"userInteractionCount" content="2""#));
    assert!(html.contains(" / porn video by Dog House Digital"));
    assert!(html.contains(r#"rel="canonical" href="https://pornsok.com/video/dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do.html""#));
    assert!(html.contains("/pornstar/madi-collins"));
    assert!(html.contains("/channel/dog-house-digital"));
    assert!(html.contains(r#"id="related_videos""#));
    assert!(html.contains("thumb vid"));
    assert!(html.contains("thumb-cover"));
    assert!(html.contains("video-preview"));
    assert!(html.contains("ttime"));
    assert!(html.contains("tview"));
    assert!(html.contains("tlike"));
    assert!(html.contains("var related_array ="));
    assert!(html.contains("counter_more_videos = 0"));
    assert!(html.contains(r#"videourl = "dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do""#));
    assert!(html.contains(r#"class="video_download meta-item""#));
    assert!(html.contains(r#">Download</a>"#));
    assert!(
        html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D")
            || html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%253D%253D")
    );
    assert!(html.contains(r#"id="download_form""#));
    assert!(
        html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D")
            || html.contains("/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%253D%253D")
    );
    assert!(html.contains(r#"id_video = 52994"#));
}

#[actix_web::test]
async fn video_detail_unknown_slug_is_not_found() {
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
        .uri("/video/definitely-missing-slug-xyz.html")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
