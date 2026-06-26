use actix_web::{web, HttpResponse, Responder};
use askama::Template;

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

pub async fn index(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    home_response(pool, None, query, "index").await
}

pub async fn home_page_num(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    home_response(pool, Some(path.into_inner()), query, "home_page_num").await
}

async fn home_response(
    pool: web::Data<DbPool>,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
    marker: &'static str,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::production();
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

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, marker))
        .content_type("text/html; charset=utf-8")
        .body(html))
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
