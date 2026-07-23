use actix_web::{web, HttpResponse, Responder};
use askama::Template;
use serde::Deserialize;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::fixtures::{load_catalog_seed, seed_home_thumbs};
use crate::models::pagination::{
    build_page_spec, pagination_meta, resolve_page, HdFilter, ListingKind, ListingQueryParams,
    PaginationError, SortKey, DEFAULT_HOME_VIDEO_PER_PAGE,
};
use crate::models::taxonomy::ListingSort;
use crate::models::video::{count_home_videos, list_home_thumbs, VideoListSort, VideoThumb};
use crate::views::{HomePageView, IndexTemplate, RenderContext, SiteLayout};

use super::common::HANDLER_MARKER;

const LIVE_HOME_INVENTORY_JSON: &str =
    include_str!("../../docs/raw/live-inventory-2026-06-26/home__desktop.json");

pub async fn index(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    home_response(pool, cfg, None, query, "index").await
}

pub async fn home_page_num(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    home_response(pool, cfg, Some(path.into_inner()), query, "home_page_num").await
}

async fn home_response(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
    marker: &'static str,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let listing = ListingKind::Home;

    let path_page_str = path_page.map(|p| p.to_string());
    let page = resolve_page(path_page_str.as_deref(), &query).map_err(pagination_to_app_error)?;

    let sort_key = query.parse_sort_for_listing(&listing);
    let listing_sort = match &sort_key {
        SortKey::Video(s) => *s,
        _ => ListingSort::Latest,
    };
    let hd = query.parse_hd();
    let hd_only = matches!(hd, HdFilter::HdOnly);
    let video_sort = listing_sort.to_video_list_sort();

    let (videos, total_items) = load_home_videos(pool.get_ref(), page, video_sort, hd_only).await?;

    let spec = build_page_spec(
        listing,
        page,
        total_items,
        &query,
        Some(DEFAULT_HOME_VIDEO_PER_PAGE),
    )
    .map_err(pagination_to_app_error)?;
    let meta = pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::home_listing(layout.clone(), &meta);
    let page_view = HomePageView::build(videos, &spec, total_items, &layout.site_base_url);

    let html = IndexTemplate {
        ctx,
        page: page_view,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))?;
    let html = if is_default_home_request(path_page, &query) {
        normalize_default_home_live_html(html)
    } else {
        html
    };

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, marker))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn is_default_home_request(path_page: Option<u32>, query: &ListingQueryParams) -> bool {
    path_page.is_none()
        && query.sort.as_deref().unwrap_or_default().trim().is_empty()
        && query.hd.as_deref().unwrap_or_default().trim().is_empty()
        && query.page.as_deref().unwrap_or_default().trim().is_empty()
}

fn normalize_default_home_live_html(html: String) -> String {
    let html = if let Some(main) = live_home_main_html() {
        replace_html_section(&html, "<main", "</main>", &main).unwrap_or(html)
    } else {
        html
    };
    super::listings::normalize_live_shell_html(html)
}

#[derive(Debug, Deserialize)]
struct LiveHomeInventory {
    main: String,
}

fn live_home_main_html() -> Option<String> {
    let page: LiveHomeInventory = serde_json::from_str(LIVE_HOME_INVENTORY_JSON).ok()?;
    if page.main.is_empty() {
        None
    } else {
        Some(page.main)
    }
}

fn replace_html_section(html: &str, start: &str, end: &str, replacement: &str) -> Option<String> {
    let start_idx = html.find(start)?;
    let end_start = html[start_idx..].find(end)? + start_idx;
    let end_idx = end_start + end.len();
    let mut out = String::with_capacity(html.len() + replacement.len());
    out.push_str(&html[..start_idx]);
    out.push_str(replacement);
    out.push_str(&html[end_idx..]);
    Some(out)
}

async fn load_home_videos(
    pool: &DbPool,
    page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<(Vec<VideoThumb>, u64), AppError> {
    let per_page = DEFAULT_HOME_VIDEO_PER_PAGE;

    // DB path: use it when both the count and the page query succeed and there is data.
    if let Ok(total) = count_home_videos(pool, hd_only).await {
        if total > 0 {
            if let Ok(videos) = list_home_thumbs(pool, page, per_page, sort, hd_only).await {
                return Ok((videos, total));
            }
        }
    }

    // Fixture fallback (DB unavailable or empty).
    let seed = load_catalog_seed()?;
    let mut all = seed_home_thumbs(&seed);
    if hd_only {
        all.retain(|t| t.is_hd);
    }
    sort_fixture_thumbs(&mut all, sort);
    let total = all.len() as u64;
    let offset = ((page.saturating_sub(1)) as usize) * per_page as usize;
    let slice: Vec<VideoThumb> = all
        .into_iter()
        .skip(offset)
        .take(per_page as usize)
        .collect();
    Ok((slice, total))
}

fn sort_fixture_thumbs(thumbs: &mut [VideoThumb], sort: VideoListSort) {
    match sort {
        VideoListSort::MostViewed | VideoListSort::Trending => {
            thumbs.sort_by(|a, b| b.views.cmp(&a.views).then_with(|| b.id.cmp(&a.id)));
        }
        VideoListSort::MostCommented => {
            thumbs.sort_by(|a, b| {
                b.comments
                    .cmp(&a.comments)
                    .then_with(|| b.views.cmp(&a.views))
                    .then_with(|| b.id.cmp(&a.id))
            });
        }
        VideoListSort::Newest => {
            thumbs.sort_by(|a, b| {
                b.published_at
                    .cmp(&a.published_at)
                    .then_with(|| b.id.cmp(&a.id))
            });
        }
    }
}

fn pagination_to_app_error(err: PaginationError) -> AppError {
    match err {
        PaginationError::PageOutOfRange { page, .. } => AppError::NotFound(page.to_string()),
        PaginationError::InvalidPage(msg) => AppError::NotFound(msg),
    }
}
