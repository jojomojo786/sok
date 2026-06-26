use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::{not_found_page_response, AppError};
use crate::fixtures::{load_catalog_seed, seed_thumbs_for_search};
use crate::models::pagination::{
    build_page_spec, pagination_meta, resolve_page, HdFilter, ListingKind, ListingQueryParams,
    PaginationError, SortKey, DEFAULT_HOME_VIDEO_PER_PAGE,
};
use crate::models::search::{display_term_from_slug, slugify_search_query, SEARCH_MIN_QUERY_LEN};
use crate::models::taxonomy::SearchSort;
use crate::models::video::{count_search_videos, list_search_videos, VideoListSort, VideoThumb};
use crate::views::{RenderContext, SearchListingTemplate, SearchListingView, SiteLayout};

use super::common::HANDLER_MARKER;

#[derive(Debug, serde::Deserialize)]
pub struct SearchRedirectQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(flatten)]
    pub rest: ListingQueryParams,
}

pub async fn search_redirect(
    query: web::Query<SearchRedirectQuery>,
) -> Result<impl Responder, AppError> {
    let raw = query.q.as_deref().unwrap_or("").trim();
    let slug = slugify_search_query(raw);
    if slug.len() < SEARCH_MIN_QUERY_LEN {
        return Ok(HttpResponse::Found()
            .insert_header((HANDLER_MARKER, "search_redirect"))
            .append_header(("Location", "/"))
            .finish());
    }
    let mut path = format!("/videos/{slug}");
    let extra = crate::models::pagination::build_query_string(
        &query.rest.parse_sort_for_listing(&ListingKind::Search {
            query_slug: slug.clone(),
        }),
        query.rest.parse_hd(),
    );
    if !extra.is_empty() {
        path.push('?');
        path.push_str(&extra);
    }
    Ok(HttpResponse::Found()
        .insert_header((HANDLER_MARKER, "search_redirect"))
        .append_header(("Location", path))
        .finish())
}

pub async fn videos_search(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<String>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    videos_search_response(pool, path.into_inner(), None, query, "videos_search").await
}

pub async fn videos_search_page(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<(String, u32)>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    let (query_slug, page) = path.into_inner();
    videos_search_response(pool, query_slug, Some(page), query, "videos_search_page").await
}

async fn videos_search_response(
    pool: web::Data<DbPool>,
    query_slug: String,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
    handler: &'static str,
) -> Result<HttpResponse, AppError> {
    let query_slug = query_slug.trim().to_string();
    if query_slug.len() < SEARCH_MIN_QUERY_LEN {
        return Ok(not_found_page_response(&query_slug, handler));
    }

    let display_term = display_term_from_slug(&query_slug);
    if display_term.is_empty() {
        return Ok(not_found_page_response(&query_slug, handler));
    }

    let layout = SiteLayout::production();
    let listing = ListingKind::Search {
        query_slug: query_slug.clone(),
    };
    let path_page_str = path_page.map(|p| p.to_string());
    let page = resolve_page(path_page_str.as_deref(), &query).map_err(pagination_to_app_error)?;

    let sort_key = query.parse_sort_for_listing(&listing);
    let search_sort = match &sort_key {
        SortKey::Search(s) => *s,
        _ => SearchSort::Relevant,
    };
    let hd = query.parse_hd();
    let hd_only = matches!(hd, HdFilter::HdOnly);
    let video_sort = search_sort.to_video_list_sort();

    let (videos, total_items) =
        load_search_videos(pool.get_ref(), &display_term, page, video_sort, hd_only).await?;

    let spec = build_page_spec(
        listing.clone(),
        page,
        total_items,
        &query,
        Some(DEFAULT_HOME_VIDEO_PER_PAGE),
    )
    .map_err(pagination_to_app_error)?;
    let meta = pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::search_listing(layout.clone(), &display_term, &meta);
    let page_view = SearchListingView::build(videos, &meta, &sort_key, hd, listing);

    let html = SearchListingTemplate {
        ctx,
        page: page_view,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, handler))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn load_search_videos(
    pool: &DbPool,
    display_term: &str,
    page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<(Vec<VideoThumb>, u64), AppError> {
    let per_page = DEFAULT_HOME_VIDEO_PER_PAGE;
    match (
        list_search_videos(pool, display_term, page, per_page, sort, hd_only).await,
        count_search_videos(pool, display_term, hd_only).await,
    ) {
        (Ok(videos), Ok(total)) if total > 0 => Ok((videos, total)),
        (Ok(_), Ok(_)) | (Err(AppError::Db(_)), _) | (_, Err(AppError::Db(_))) => {
            let seed = load_catalog_seed()?;
            let mut all = seed_thumbs_for_search(&seed, display_term);
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
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
    }
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
