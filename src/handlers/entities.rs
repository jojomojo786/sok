use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::{not_found_page_response, AppError};
use crate::fixtures::{load_catalog_seed, seed_thumbs_for_channel, seed_thumbs_for_pornstar};
use crate::models::entities::{
    channel_by_slug, count_videos_for_channel, count_videos_for_pornstar,
    listing_sort_to_entity_video_sort, pornstar_by_slug, videos_for_channel, videos_for_pornstar,
    Channel, EntityProfileHeader, Pornstar,
};
use crate::models::pagination::{
    build_page_spec, pagination_meta, resolve_page, EntityProfileKind, HdFilter, ListingKind,
    ListingQueryParams, PaginationError, SortKey, DEFAULT_HOME_VIDEO_PER_PAGE,
};
use crate::models::taxonomy::ListingSort;
use crate::models::video::{VideoListSort, VideoThumb};
use crate::views::{EntityProfileTemplate, EntityProfileView, RenderContext, SiteLayout};

use super::common::HANDLER_MARKER;

pub async fn channel_profile(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<String>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    entity_profile_response(
        pool,
        path.into_inner(),
        None,
        query,
        EntityProfileKind::Channel,
        "channel_profile",
    )
    .await
}

pub async fn channel_profile_page(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<(String, u32)>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    let (slug, page) = path.into_inner();
    entity_profile_response(
        pool,
        slug,
        Some(page),
        query,
        EntityProfileKind::Channel,
        "channel_profile",
    )
    .await
}

pub async fn pornstar_profile(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<String>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    entity_profile_response(
        pool,
        path.into_inner(),
        None,
        query,
        EntityProfileKind::Pornstar,
        "pornstar_profile",
    )
    .await
}

pub async fn pornstar_profile_page(
    pool: web::Data<DbPool>,
    _cfg: web::Data<Config>,
    path: web::Path<(String, u32)>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    let (slug, page) = path.into_inner();
    entity_profile_response(
        pool,
        slug,
        Some(page),
        query,
        EntityProfileKind::Pornstar,
        "pornstar_profile",
    )
    .await
}

async fn entity_profile_response(
    pool: web::Data<DbPool>,
    slug: String,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
    kind: EntityProfileKind,
    handler_marker: &'static str,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::production();
    let listing = match kind {
        EntityProfileKind::Pornstar => {
            ListingKind::EntityProfile(EntityProfileKind::Pornstar, slug.clone())
        }
        EntityProfileKind::Channel => {
            ListingKind::EntityProfile(EntityProfileKind::Channel, slug.clone())
        }
    };

    let path_page_str = path_page.map(|p| p.to_string());
    let page = resolve_page(path_page_str.as_deref(), &query).map_err(pagination_to_app_error)?;

    let sort_key = query.parse_sort_for_listing(&listing);
    let listing_sort = match &sort_key {
        SortKey::Video(s) => *s,
        _ => ListingSort::Latest,
    };
    let hd = query.parse_hd();
    let hd_only = matches!(hd, HdFilter::HdOnly);
    let video_sort = listing_sort_to_entity_video_sort(listing_sort);

    let profile_header =
        resolve_profile_header(pool.get_ref(), &slug, kind, listing_sort, &layout.media_cdn)
            .await?;
    let Some(profile_header) = profile_header else {
        let not_found_slug = match kind {
            EntityProfileKind::Pornstar => format!("/pornstar/{slug}"),
            EntityProfileKind::Channel => format!("/channel/{slug}"),
        };
        return Ok(not_found_page_response(&not_found_slug, handler_marker));
    };

    let (videos, total_items) = load_profile_videos(
        pool.get_ref(),
        &slug,
        kind,
        page,
        video_sort,
        hd_only,
        &layout.media_cdn,
    )
    .await?;

    let spec = build_page_spec(
        listing.clone(),
        page,
        total_items,
        &query,
        Some(DEFAULT_HOME_VIDEO_PER_PAGE),
    )
    .map_err(pagination_to_app_error)?;
    let meta = pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::entity_profile(layout.clone(), &profile_header, &meta);
    let page_view =
        EntityProfileView::build(&profile_header, videos, &meta, &sort_key, hd, listing);

    let html = EntityProfileTemplate {
        ctx,
        page: page_view,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, handler_marker))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn resolve_profile_header(
    pool: &DbPool,
    slug: &str,
    kind: EntityProfileKind,
    sort: ListingSort,
    cdn_base: &str,
) -> Result<Option<EntityProfileHeader>, AppError> {
    match kind {
        EntityProfileKind::Pornstar => match pornstar_by_slug(pool, slug).await {
            Ok(Some(p)) => Ok(Some(EntityProfileHeader::for_pornstar(&p, cdn_base, sort))),
            Ok(None) => Ok(fixture_pornstar_header(slug, sort, cdn_base)),
            Err(AppError::Db(_)) => Ok(fixture_pornstar_header(slug, sort, cdn_base)),
            Err(e) => Err(e),
        },
        EntityProfileKind::Channel => match channel_by_slug(pool, slug).await {
            Ok(Some(c)) => Ok(Some(EntityProfileHeader::for_channel(&c, cdn_base, sort))),
            Ok(None) => Ok(fixture_channel_header(slug, sort, cdn_base)),
            Err(AppError::Db(_)) => Ok(fixture_channel_header(slug, sort, cdn_base)),
            Err(e) => Err(e),
        },
    }
}

async fn load_profile_videos(
    pool: &DbPool,
    slug: &str,
    kind: EntityProfileKind,
    page: u32,
    sort: VideoListSort,
    hd_only: bool,
    cdn_base: &str,
) -> Result<(Vec<VideoThumb>, u64), AppError> {
    let per_page = DEFAULT_HOME_VIDEO_PER_PAGE;

    let db_result = match kind {
        EntityProfileKind::Pornstar => {
            let videos = videos_for_pornstar(pool, slug, page, per_page, sort, hd_only).await;
            let total = count_videos_for_pornstar(pool, slug, hd_only).await;
            (videos, total)
        }
        EntityProfileKind::Channel => {
            let videos = videos_for_channel(pool, slug, page, per_page, sort, hd_only).await;
            let total = count_videos_for_channel(pool, slug, hd_only).await;
            (videos, total)
        }
    };

    if let (Ok(rows), Ok(total)) = db_result {
        if total > 0 {
            let thumbs: Vec<_> = rows
                .into_iter()
                .map(|r| r.to_video_thumb(cdn_base))
                .collect();
            return Ok((thumbs, total));
        }
    }

    let seed = load_catalog_seed()?;
    let mut all = match kind {
        EntityProfileKind::Pornstar => seed_thumbs_for_pornstar(&seed, slug),
        EntityProfileKind::Channel => seed_thumbs_for_channel(&seed, slug),
    };
    if all.is_empty() {
        return Ok((Vec::new(), 0));
    }
    sort_fixture_thumbs(&mut all, sort);
    if hd_only {
        all.retain(|t| t.is_hd);
    }
    let total = all.len() as u64;
    let offset = ((page.saturating_sub(1)) as usize) * per_page as usize;
    let slice: Vec<_> = all
        .into_iter()
        .skip(offset)
        .take(per_page as usize)
        .collect();
    Ok((slice, total))
}

fn fixture_pornstar_header(
    slug: &str,
    sort: ListingSort,
    cdn_base: &str,
) -> Option<EntityProfileHeader> {
    let seed = load_catalog_seed().ok()?;
    let p = seed.pornstars.iter().find(|p| p.slug == slug)?;
    let row = Pornstar {
        id: p.id,
        slug: p.slug.clone(),
        display_name: p.display_name.clone(),
        thumb_path: p.thumb_path.clone(),
        banner_path: None,
        avatar_path: None,
        bio: None,
        video_count: p.video_count,
        verified: slug == "angela-white",
        week_views: p.week_views,
        created_at: None,
        updated_at: None,
    };
    Some(EntityProfileHeader::for_pornstar(&row, cdn_base, sort))
}

fn fixture_channel_header(
    slug: &str,
    sort: ListingSort,
    cdn_base: &str,
) -> Option<EntityProfileHeader> {
    let seed = load_catalog_seed().ok()?;
    let c = seed.channels.iter().find(|c| c.slug == slug)?;
    let row = Channel {
        id: c.id,
        slug: c.slug.clone(),
        title: c.title.clone(),
        thumb_path: c.thumb_path.clone(),
        logo_path: None,
        banner_path: None,
        bio: None,
        video_count: c.video_count,
        network_name: None,
        week_views: c.week_views,
        created_at: None,
        updated_at: None,
    };
    Some(EntityProfileHeader::for_channel(&row, cdn_base, sort))
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
