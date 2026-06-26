//! Consolidated AJAX response-shape contracts for fields consumed by `static/js/main.min.js`.
//!
//! sok-replica.9.2 — complements per-endpoint tests with a single map of frontend-critical keys.

mod common;

use actix_web::http::StatusCode;
use common::ajax::{
    assert_object_has_only_keys, handler_marker, json_body, post_form, post_optional_body,
};
use sok::fixtures::{load_catalog_seed, search_categories_and_tags_from_seed};
use sok::models::entities::DEFAULT_MEDIA_CDN;
use sok::models::entity_page_search::{
    search_entities_for_page_from_seed, EntityPageSearchType, ENTITY_PAGE_SEARCH_LIMIT,
};
use sok::models::search_help::{search_help_from_seed, SEARCH_HELP_GROUP_LIMIT};
use sok::models::taxonomy::TagRow;
use sok::views::build_update_tags_response;

/// Top-level keys read by `makeResult` / stale guard in header search.
const SEARCH_HELP_TOP_KEYS: &[&str] = &["pornstars", "channels", "videos", "search_text"];

/// Keys appended into `#ajax_tags` and preload queue (`refresh_tags`).
const UPDATE_TAGS_KEYS: &[&str] = &["html", "preload_before", "preload_after", "preload_array"];

/// In-page entity search card keys (`#search-page-input` handlers).
const ENTITY_PAGE_ITEM_KEYS: &[&str] = &["url", "thumb", "orig_name", "count_videos"];

#[actix_web::test]
async fn live_search_help_json_matches_main_min_js_surface() {
    let resp = post_form("/ajax/search_help", "text=milf").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(handler_marker(&resp), Some("search_help"));

    let v = json_body(resp).await;
    assert_object_has_only_keys(&v, SEARCH_HELP_TOP_KEYS);
    assert_eq!(v["search_text"].as_str(), Some("milf"));

    for item in v["pornstars"].as_array().unwrap() {
        for key in ["url_pornstar", "name", "orig_name", "thumb", "count_videos"] {
            assert!(item.get(key).is_some(), "pornstar item missing {key}");
        }
    }
    for item in v["channels"].as_array().unwrap() {
        for key in ["url", "orig_name", "rus_name", "thumb", "count_videos"] {
            assert!(item.get(key).is_some(), "channel item missing {key}");
        }
    }
    for item in v["videos"].as_array().unwrap() {
        for key in ["url", "title", "thumb", "widethumb"] {
            assert!(item.get(key).is_some(), "video item missing {key}");
        }
    }
}

#[actix_web::test]
async fn live_search_cats_tags_queries_json_matches_main_min_js_surface() {
    let resp = post_form("/ajax/search_cats_tags_queries", "text=milf").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(handler_marker(&resp), Some("search_cats_tags_queries"));

    let v = json_body(resp).await;
    assert!(v.get("search_text").and_then(|x| x.as_str()).is_some());
    let items = v["items"].as_array().expect("items array");
    for item in items {
        for key in ["id", "name", "url"] {
            assert!(
                item.get(key).and_then(|x| x.as_str()).is_some(),
                "missing {key}"
            );
        }
    }
}

#[actix_web::test]
async fn live_update_tags_json_matches_refresh_tags_contract() {
    let resp = post_form("/ajax/update_tags", "").await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(handler_marker(&resp), Some("update_tags"));

    let v = json_body(resp).await;
    assert_object_has_only_keys(&v, UPDATE_TAGS_KEYS);
    assert!(v["html"].as_str().unwrap().contains("fa-tag"));
    assert!(v["preload_array"].is_array());
}

#[actix_web::test]
async fn live_update_widget_html_fragments_expose_jquery_injection_hooks() {
    for (path, marker, payload, hooks) in [
        (
            "/ajax/update_pornstars",
            "update_pornstars",
            "",
            &["class=\"thumb cat\"", "count-videos", "/pornstar/"][..],
        ),
        (
            "/ajax/update_channels",
            "update_channels",
            "",
            &["class=\"thumb cat\"", "/channel/"][..],
        ),
        (
            "/ajax/update_watching_now",
            "update_watching_now",
            "order_by=week_views",
            &["class=\"thumb vid\"", "data-video=", "fa-eye"][..],
        ),
        (
            "/ajax/update_newest_videos",
            "update_newest_videos",
            "video_id=1&offset=0&count=12",
            &["class=\"thumb vid\"", "/video/"][..],
        ),
    ] {
        let resp = post_form(path, payload).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(handler_marker(&resp), Some(marker));
        let body = String::from_utf8(actix_web::test::read_body(resp).await.to_vec()).unwrap();
        for hook in hooks {
            assert!(body.contains(hook), "{path} missing hook {hook}");
        }
    }
}

#[actix_web::test]
async fn live_add_hit_favourite_matches_jquery_post_expectations() {
    let resp = post_optional_body("/ajax/add_hit/favourite", None).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(handler_marker(&resp), Some("add_hit_favourite"));
    let body = actix_web::test::read_body(resp).await;
    assert!(body.is_empty(), "favourite hit should return empty body");
}

#[actix_web::test]
async fn live_entity_search_json_matches_in_page_card_contract() {
    for (path, marker, query) in [
        ("/ajax/search_pstars", "search_pornstars", "mil"),
        ("/ajax/search_channels", "search_channels", "bra"),
    ] {
        let resp = post_form(path, &format!("text={query}")).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(handler_marker(&resp), Some(marker));

        let v = json_body(resp).await;
        assert_eq!(v["search_text"].as_str(), Some(query));
        let items = v["items"].as_array().expect("items");
        for item in items {
            for key in ENTITY_PAGE_ITEM_KEYS {
                assert!(item.get(*key).is_some(), "{path} item missing {key}");
            }
            assert!(item["url"].as_str().unwrap().starts_with('/'));
            assert!(item["thumb"].as_str().unwrap().starts_with("http"));
        }
    }
}

#[actix_web::test]
async fn live_more_videos_and_comments_shapes_used_on_video_page() {
    use sok::models::video::fixtures::DOG_HOUSE_SLUG;

    let related = post_form("/ajax/more_videos_3", &format!("videourl={DOG_HOUSE_SLUG}")).await;
    assert_eq!(related.status(), StatusCode::OK);
    let batches = json_body(related).await;
    assert!(batches.is_array());
    if let Some(batch) = batches
        .as_array()
        .and_then(|b| b.first())
        .and_then(|b| b.as_array())
    {
        if let Some(item) = batch.first() {
            for key in [
                "url",
                "title",
                "thumb",
                "preview_mini",
                "duration",
                "views",
                "str_views",
                "rate",
                "widethumb",
            ] {
                assert!(item.get(key).is_some(), "more_videos item missing {key}");
            }
        }
    }

    let comments = post_form("/ajax/comments", "name=Ada&msg=Hello&vid=52994").await;
    let v = json_body(comments).await;
    assert!(v.get("result").and_then(|x| x.as_str()).is_some());

    let more = post_form("/ajax/more_comments", &format!("videourl={DOG_HOUSE_SLUG}")).await;
    let v = json_body(more).await;
    assert!(v.get("comments").and_then(|x| x.as_str()).is_some());
}

#[test]
fn fixture_search_help_serializes_only_frontend_consumed_groups() {
    let seed = load_catalog_seed().expect("seed");
    let resp = search_help_from_seed(&seed, "milf", SEARCH_HELP_GROUP_LIMIT);
    let v = serde_json::to_value(&resp).unwrap();
    assert_object_has_only_keys(&v, SEARCH_HELP_TOP_KEYS);
    assert_eq!(v["search_text"], "milf");
}

#[test]
fn fixture_cats_tags_search_items_expose_name_and_url_for_dom_builder() {
    let seed = load_catalog_seed().expect("seed");
    let resp = search_categories_and_tags_from_seed(&seed, "milf", 50);
    assert_eq!(resp.search_text, "milf");
    for item in &resp.items {
        assert!(!item.id.is_empty());
        assert!(!item.name.is_empty());
        assert!(item.url.starts_with('/'));
    }
}

#[test]
fn fixture_entity_page_search_items_match_card_template_keys() {
    let seed = load_catalog_seed().expect("seed");
    let resp = search_entities_for_page_from_seed(
        &seed,
        EntityPageSearchType::Pornstars,
        "ang",
        ENTITY_PAGE_SEARCH_LIMIT,
        DEFAULT_MEDIA_CDN,
    );
    assert!(!resp.items.is_empty());
    for item in &resp.items {
        assert!(item.url.starts_with("/pornstar/"));
        assert!(item.thumb.starts_with("http"));
        assert!(!item.orig_name.is_empty());
    }
    let json = serde_json::to_value(&resp).unwrap();
    assert!(json.get("search_text").is_some());
    assert!(json.get("items").is_some());
}

#[test]
fn update_tags_builder_exposes_preload_fields_for_process_preloads() {
    let tags = vec![TagRow {
        id: 1,
        slug: "milf".into(),
        display_name: "MILF".into(),
        description: None,
        thumb_url: None,
        video_count: 10,
        weekly_views: 0,
        is_active: true,
    }];
    let resp = build_update_tags_response(&tags);
    let v = serde_json::to_value(&resp).unwrap();
    assert_object_has_only_keys(&v, UPDATE_TAGS_KEYS);
    assert!(resp.html.contains("fa-tag"));
    assert!(!resp.preload_before.is_empty());
    assert_eq!(resp.preload_array, vec!["milf".to_string()]);
}
