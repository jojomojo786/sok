use actix_web::{web, HttpResponse, Responder};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::fixtures::{
    load_catalog_seed, search_categories_and_tags_from_seed, seed_home_thumbs,
    seed_top_channels_week, seed_top_pornstars_week, seed_top_viewed_tags,
};
use crate::logging::log_ajax_db_fallback;
use crate::models::comment_store::{
    list_comments_for_video_slug, parse_video_id_param, render_comments_html_fragment,
    submit_comment_for_video_id, validation_error_message, CommentSubmitResponse,
    MoreCommentsResponse, COMMENTS_INITIAL_LIMIT, COMMENTS_MORE_BATCH_SIZE,
};
use crate::models::entities::{list_top_channels_week, list_top_pornstars_week, EntityIndexCard};
use crate::models::entity_page_search::{
    entity_page_search_fallback, search_entities_for_page, EntityPageSearchType,
    ENTITY_PAGE_SEARCH_LIMIT,
};
use crate::models::metrics::record_favourite_hit_best_effort;
use crate::models::search_help::{
    search_help_from_db, search_help_from_seed, SearchHelpResponse, SEARCH_HELP_GROUP_LIMIT,
};
use crate::models::taxonomy::{
    list_top_viewed_tags, search_categories_and_tags, CatsTagsSearchResponse, TagRow,
};
use crate::models::video::fixtures::related_fixture_batches_for_slug;
use crate::models::video::{
    list_home_thumbs, list_watching_now_thumbs, normalize_video_slug, record_vote_for_video,
    related_ajax_batches_for_video, video_likes_percent_by_id, VideoListSort, VideoThumb,
    VoteDirection,
};
use crate::views::{
    build_update_tags_response, render_channels_widget, render_newest_videos_widget,
    render_pornstars_widget, render_watching_now_widget, HOME_WIDGET_CHANNELS_LIMIT,
    HOME_WIDGET_NEWEST_DEFAULT_COUNT, HOME_WIDGET_NEWEST_MAX_COUNT, HOME_WIDGET_PORNSTARS_LIMIT,
    HOME_WIDGET_TAGS_LIMIT, HOME_WIDGET_WATCHING_NOW_LIMIT,
};

use super::common::{stub_response, HANDLER_MARKER};

#[derive(Debug, Deserialize)]
pub struct SearchTextForm {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct VideoUrlForm {
    pub videourl: String,
}

#[derive(Debug, Deserialize)]
pub struct CommentSubmitForm {
    pub name: String,
    pub msg: String,
    pub vid: String,
}

#[derive(Debug, Deserialize)]
pub struct VoteForm {
    pub id_video: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VoteResponse {
    pub raiting: u8,
    pub msg: String,
}

const CATS_TAGS_SEARCH_LIMIT: u32 = 80;

/// Build a JSON-bodied response that matches live pornsok.com transport for the
/// mirrored jQuery 3.3.1 AJAX endpoints. Production serves these JSON payloads
/// as `text/html; charset=UTF-8` with `X-Content-Type-Options: nosniff` so the
/// client's `$.parseJSON(responseText)` receives a raw string instead of an
/// object jQuery already parsed. The body contract is unchanged; only the
/// transport headers differ from `application/json`.
fn live_json_response(handler: &'static str, body: String) -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, handler))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .content_type("text/html; charset=utf-8")
        .body(body)
}

pub async fn more_videos_3(
    pool: web::Data<DbPool>,
    form: web::Form<VideoUrlForm>,
) -> Result<impl Responder, AppError> {
    let slug = normalize_video_slug(&form.videourl);
    let batches = match related_ajax_batches_for_video(pool.get_ref(), &slug).await {
        Ok(batches) if !batches.is_empty() => batches,
        Ok(_) => related_fixture_batches_for_slug(&slug),
        Err(AppError::Db(e)) => {
            log_ajax_db_fallback("more_videos_3", &e);
            related_fixture_batches_for_slug(&slug)
        }
        Err(e) => return Err(e),
    };

    let body = serde_json::to_string(&batches)
        .map_err(|e| AppError::Internal(format!("more_videos_3 json: {e}")))?;

    Ok(live_json_response("more_videos_3", body))
}

pub async fn add_hit_more_videos() -> HttpResponse {
    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "add_hit_more_videos"))
        .content_type("text/plain; charset=utf-8")
        .body("ok")
}

pub async fn add_hit_favourite(pool: web::Data<DbPool>) -> HttpResponse {
    record_favourite_hit_best_effort(pool.get_ref()).await;
    HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "add_hit_favourite"))
        .body("")
}

pub async fn add_vote_v3(
    pool: web::Data<DbPool>,
    form: web::Form<VoteForm>,
) -> Result<impl Responder, AppError> {
    let Some(video_id) = parse_video_id_param(&form.id_video) else {
        let body = serde_json::to_string(&VoteResponse {
            raiting: 0,
            msg: "Unknown video.".into(),
        })
        .map_err(|e| AppError::Internal(format!("add_vote_v3 json: {e}")))?;
        return Ok(live_json_response("add_vote_v3", body));
    };

    let raiting = match VoteDirection::from_status(&form.status) {
        Some(direction) => record_vote_for_video(pool.get_ref(), video_id, direction).await,
        // Unrecognized status: don't persist, just report the current rating.
        None => video_likes_percent_by_id(pool.get_ref(), video_id).await,
    }
    .unwrap_or(0);
    let msg = vote_ack_message(&form.status);
    let body = serde_json::to_string(&VoteResponse { raiting, msg })
        .map_err(|e| AppError::Internal(format!("add_vote_v3 json: {e}")))?;

    Ok(live_json_response("add_vote_v3", body))
}

fn vote_ack_message(status: &str) -> String {
    match status.trim() {
        "1" | "up" | "like" => "Thanks for your vote.".into(),
        "0" | "-1" | "down" | "dislike" | "unlike" => "Thanks for your feedback.".into(),
        _ => "Vote recorded locally.".into(),
    }
}

pub async fn ajax_reserved() -> impl Responder {
    stub_response("ajax")
}

pub async fn search_cats_tags_queries(
    pool: web::Data<DbPool>,
    form: web::Form<SearchTextForm>,
) -> Result<impl Responder, AppError> {
    let text = &form.text;
    let response =
        match search_categories_and_tags(pool.get_ref(), text, CATS_TAGS_SEARCH_LIMIT).await {
            Ok(resp) if should_use_fixture_fallback(&resp, text) => fallback_search(text)?,
            Ok(resp) => resp,
            Err(AppError::Db(e)) => {
                log_ajax_db_fallback("search_cats_tags_queries", &e);
                fallback_search(text)?
            }
            Err(e) => return Err(e),
        };

    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("search_cats_tags_queries json: {e}")))?;

    Ok(live_json_response("search_cats_tags_queries", body))
}

fn should_use_fixture_fallback(resp: &CatsTagsSearchResponse, text: &str) -> bool {
    text.trim().len() >= 2 && resp.items.is_empty()
}

fn fallback_search(text: &str) -> Result<CatsTagsSearchResponse, AppError> {
    let seed = load_catalog_seed()?;
    Ok(search_categories_and_tags_from_seed(
        &seed,
        text,
        CATS_TAGS_SEARCH_LIMIT,
    ))
}

pub async fn search_help(
    pool: web::Data<DbPool>,
    form: web::Form<SearchTextForm>,
) -> Result<impl Responder, AppError> {
    let text = &form.text;
    let response = match search_help_from_db(pool.get_ref(), text, SEARCH_HELP_GROUP_LIMIT).await {
        Ok(resp) if resp.is_empty() => search_help_fallback(text)?,
        Ok(resp) => resp,
        Err(AppError::Db(e)) => {
            log_ajax_db_fallback("search_help", &e);
            search_help_fallback(text)?
        }
        Err(e) => return Err(e),
    };

    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("search_help json: {e}")))?;

    // Live pornsok.com serves this JSON-bodied autocomplete response with a
    // `text/html` content-type and `X-Content-Type-Options: nosniff`. The
    // mirrored jQuery 3.3.1 header search calls `$.parseJSON(responseText)`,
    // which expects a raw string. Returning `application/json` would make
    // jQuery auto-parse the body into an object, so `$.parseJSON` then
    // double-parses and yields null, leaving the autocomplete empty. Match the
    // live content-type to keep that client path working in-browser.
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "search_help"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .content_type("text/html; charset=utf-8")
        .body(body))
}

fn search_help_fallback(text: &str) -> Result<SearchHelpResponse, AppError> {
    let seed = load_catalog_seed()?;
    Ok(search_help_from_seed(&seed, text, SEARCH_HELP_GROUP_LIMIT))
}

#[derive(Debug, Deserialize)]
pub struct EntitySearchPath {
    search_type: String,
}

pub async fn search_entity_page(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<EntitySearchPath>,
    form: web::Form<SearchTextForm>,
) -> Result<impl Responder, AppError> {
    let media_cdn = cfg.media_cdn.as_str();
    let Some(search_type) = EntityPageSearchType::parse(&path.search_type) else {
        return Ok(HttpResponse::NotFound()
            .insert_header((HANDLER_MARKER, "ajax"))
            .content_type("text/plain; charset=utf-8")
            .body("unsupported search type"));
    };

    let text = &form.text;
    let response = match search_entities_for_page(
        pool.get_ref(),
        search_type,
        text,
        ENTITY_PAGE_SEARCH_LIMIT,
        media_cdn,
    )
    .await
    {
        Ok(resp) if should_use_entity_fixture_fallback(&resp, text) => {
            entity_page_search_fallback(search_type, text, ENTITY_PAGE_SEARCH_LIMIT, media_cdn)?
        }
        Ok(resp) => resp,
        Err(AppError::Db(e)) => {
            log_ajax_db_fallback(search_type.handler_marker(), &e);
            entity_page_search_fallback(search_type, text, ENTITY_PAGE_SEARCH_LIMIT, media_cdn)?
        }
        Err(e) => return Err(e),
    };

    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("{} json: {e}", search_type.handler_marker())))?;

    Ok(live_json_response(search_type.handler_marker(), body))
}

fn should_use_entity_fixture_fallback(
    resp: &crate::models::entity_page_search::EntityPageSearchResponse,
    text: &str,
) -> bool {
    text.trim().len() >= 2 && resp.items.is_empty()
}

#[derive(Debug, Deserialize)]
pub struct WatchingNowForm {
    pub order_by: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NewestVideosForm {
    pub video_id: Option<String>,
    pub offset: Option<u32>,
    pub count: Option<u32>,
}

pub async fn update_pornstars(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let media_cdn = cfg.media_cdn.as_str();
    let cards = load_widget_pornstars(pool.get_ref(), HOME_WIDGET_PORNSTARS_LIMIT).await?;
    let html = render_pornstars_widget(&cards, media_cdn);
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "update_pornstars"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub async fn update_channels(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let media_cdn = cfg.media_cdn.as_str();
    let cards = load_widget_channels(pool.get_ref(), HOME_WIDGET_CHANNELS_LIMIT).await?;
    let html = render_channels_widget(&cards, media_cdn);
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "update_channels"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub async fn update_tags(pool: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    let tags = load_widget_tags(pool.get_ref(), HOME_WIDGET_TAGS_LIMIT).await?;
    let response = build_update_tags_response(&tags);
    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("update_tags json: {e}")))?;
    Ok(live_json_response("update_tags", body))
}

pub async fn update_watching_now(
    pool: web::Data<DbPool>,
    form: web::Form<WatchingNowForm>,
) -> Result<impl Responder, AppError> {
    let _order_by = form.order_by.as_deref().unwrap_or("week_views");
    let videos = load_widget_watching_now(pool.get_ref(), HOME_WIDGET_WATCHING_NOW_LIMIT).await?;
    let html = render_watching_now_widget(&videos);
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "update_watching_now"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub async fn update_newest_videos(
    pool: web::Data<DbPool>,
    form: web::Form<NewestVideosForm>,
) -> Result<impl Responder, AppError> {
    let _video_id = form.video_id.as_deref();
    let offset = form.offset.unwrap_or(0);
    let count = form
        .count
        .unwrap_or(HOME_WIDGET_NEWEST_DEFAULT_COUNT)
        .clamp(1, HOME_WIDGET_NEWEST_MAX_COUNT);
    let page = offset / count + 1;
    let videos = load_widget_newest_videos(pool.get_ref(), page, count).await?;
    let html = render_newest_videos_widget(&videos);
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "update_newest_videos"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn load_widget_pornstars(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<EntityIndexCard>, AppError> {
    match list_top_pornstars_week(pool, limit).await {
        Ok(cards) if !cards.is_empty() => Ok(cards),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(seed_top_pornstars_week(&seed, limit))
        }
        Err(e) => Err(e),
    }
}

async fn load_widget_channels(pool: &DbPool, limit: u32) -> Result<Vec<EntityIndexCard>, AppError> {
    match list_top_channels_week(pool, limit).await {
        Ok(cards) if !cards.is_empty() => Ok(cards),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(seed_top_channels_week(&seed, limit))
        }
        Err(e) => Err(e),
    }
}

async fn load_widget_tags(pool: &DbPool, limit: u32) -> Result<Vec<TagRow>, AppError> {
    match list_top_viewed_tags(pool, limit).await {
        Ok(tags) if !tags.is_empty() => Ok(tags),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(seed_top_viewed_tags(&seed, limit))
        }
        Err(e) => Err(e),
    }
}

async fn load_widget_watching_now(pool: &DbPool, limit: u32) -> Result<Vec<VideoThumb>, AppError> {
    match list_watching_now_thumbs(pool, limit).await {
        Ok(videos) if !videos.is_empty() => Ok(videos),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            let mut all = seed_home_thumbs(&seed);
            all.sort_by(|a, b| {
                b.views
                    .cmp(&a.views)
                    .then_with(|| b.published_at.cmp(&a.published_at))
                    .then_with(|| b.id.cmp(&a.id))
            });
            all.truncate(limit as usize);
            Ok(all)
        }
        Err(e) => Err(e),
    }
}

async fn load_widget_newest_videos(
    pool: &DbPool,
    page: u32,
    count: u32,
) -> Result<Vec<VideoThumb>, AppError> {
    match list_home_thumbs(pool, page, count, VideoListSort::Newest, false).await {
        Ok(videos) if !videos.is_empty() => Ok(videos),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            let mut all = seed_home_thumbs(&seed);
            all.sort_by(|a, b| {
                b.published_at
                    .cmp(&a.published_at)
                    .then_with(|| b.views.cmp(&a.views))
                    .then_with(|| b.id.cmp(&a.id))
            });
            let offset = ((page.saturating_sub(1)) as usize) * count as usize;
            Ok(all.into_iter().skip(offset).take(count as usize).collect())
        }
        Err(e) => Err(e),
    }
}

pub async fn post_comments(
    pool: web::Data<DbPool>,
    form: web::Form<CommentSubmitForm>,
) -> Result<impl Responder, AppError> {
    let Some(video_id) = parse_video_id_param(&form.vid) else {
        let body = serde_json::to_string(&CommentSubmitResponse::err("Unknown video."))
            .map_err(|e| AppError::Internal(format!("post_comments json: {e}")))?;
        return Ok(HttpResponse::Ok()
            .insert_header((HANDLER_MARKER, "post_comments"))
            .content_type("application/json; charset=utf-8")
            .body(body));
    };

    let response =
        match submit_comment_for_video_id(pool.get_ref(), video_id, &form.name, &form.msg).await {
            Ok(_) => CommentSubmitResponse::ok("Thank you! Your comment was posted."),
            Err(err) => CommentSubmitResponse::err(validation_error_message(&err)),
        };

    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("post_comments json: {e}")))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "post_comments"))
        .content_type("application/json; charset=utf-8")
        .body(body))
}

pub async fn post_more_comments(
    pool: web::Data<DbPool>,
    form: web::Form<VideoUrlForm>,
) -> Result<impl Responder, AppError> {
    let slug = normalize_video_slug(&form.videourl);
    let comments = list_comments_for_video_slug(
        pool.get_ref(),
        &slug,
        COMMENTS_MORE_BATCH_SIZE,
        COMMENTS_INITIAL_LIMIT,
    )
    .await?;
    let response = MoreCommentsResponse {
        comments: render_comments_html_fragment(&comments),
    };
    let body = serde_json::to_string(&response)
        .map_err(|e| AppError::Internal(format!("post_more_comments json: {e}")))?;
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "post_more_comments"))
        .content_type("application/json; charset=utf-8")
        .body(body))
}
