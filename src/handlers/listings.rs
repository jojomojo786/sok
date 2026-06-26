use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::{not_found_page_response, AppError};
use crate::fixtures::{
    load_catalog_seed, seed_channel_cards, seed_pornstar_cards, seed_thumbs_for_category,
};
use crate::models::entities::{
    list_channels_index, list_pornstars_index, EntityIndexCard, EntityListSort,
    ENTITY_INDEX_PAGE_SIZE,
};
use crate::models::pagination::{
    build_page_spec, pagination_meta, resolve_page, EntityIndexKind, EntitySortKey, HdFilter,
    ListingKind, ListingQueryParams, PaginationError, SortKey, DEFAULT_HOME_VIDEO_PER_PAGE,
};
use crate::models::taxonomy::{
    get_category_by_slug, get_tag_by_slug, list_tags_for_index, CategoryRow, ListingSlugKind,
    ListingSort, TagRow, TaxonomyListingHeader,
};
use crate::models::video::{
    count_videos_for_category, count_videos_for_tag, list_thumbs_for_category, list_thumbs_for_tag,
    VideoThumb,
};
use crate::views::load_categories_page_data;
use crate::views::ChannelsIndexView;
use crate::views::PornstarsIndexView;
use crate::views::{
    CategoriesTemplate, ChannelsTemplate, PornstarsTemplate, RenderContext, SiteLayout,
};
use crate::views::{SlugListingTemplate, SlugListingView, TagsHubView, TagsTemplate};

use super::common::HANDLER_MARKER;

pub async fn categories(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let page = load_categories_page_data(pool.get_ref(), &layout).await?;
    let ctx = RenderContext::categories_index(layout);
    let html = CategoriesTemplate {
        ctx,
        categories: page.categories,
        top_tags: page.top_tags,
        tag_preload_slugs: page.tag_preload_slugs,
        top_pornstars: page.top_pornstars,
    }
    .render()
    .unwrap();
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "categories"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

pub async fn pornstars_list(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    pornstars_index_response(pool, cfg, None, query).await
}

pub async fn pornstars_list_page(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    pornstars_index_response(pool, cfg, Some(path.into_inner()), query).await
}

async fn pornstars_index_response(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let listing = ListingKind::EntityIndex(EntityIndexKind::Pornstars);
    let path_page_str = path_page.map(|p| p.to_string());
    let path_page_ref = path_page_str.as_deref();

    let (cards, total_items) = load_pornstars_page(pool.get_ref(), &query, path_page_ref).await?;
    let page = crate::models::pagination::resolve_page(path_page_ref, &query)
        .map_err(pagination_to_app_error)?;
    let spec = crate::models::pagination::build_page_spec(
        listing.clone(),
        page,
        total_items,
        &query,
        Some(ENTITY_INDEX_PAGE_SIZE),
    )
    .map_err(pagination_to_app_error)?;
    let meta =
        crate::models::pagination::pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::pornstars_index(layout.clone(), &meta);
    let pornstars = PornstarsIndexView::build(
        cards,
        &meta,
        &query.parse_sort_for_listing(&ListingKind::EntityIndex(EntityIndexKind::Pornstars)),
        &layout.media_cdn,
    );

    let html = PornstarsTemplate { ctx, pornstars }
        .render()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "pornstars"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn load_pornstars_page(
    pool: &DbPool,
    query: &ListingQueryParams,
    path_page: Option<&str>,
) -> Result<(Vec<EntityIndexCard>, u64), AppError> {
    let listing = ListingKind::EntityIndex(EntityIndexKind::Pornstars);
    let sort_key = query.parse_sort_for_listing(&listing);
    let entity_sort = entity_list_sort_from_sort_key(&sort_key);
    let page = crate::models::pagination::resolve_page(path_page, query)
        .map_err(pagination_to_app_error)?;

    match list_pornstars_index(pool, entity_sort, page, ENTITY_INDEX_PAGE_SIZE).await {
        Ok(page_data) if !page_data.items.is_empty() => Ok((page_data.items, page_data.total)),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            let mut cards = seed_pornstar_cards(&seed);
            sort_fixture_pornstars(&mut cards, &sort_key);
            let total = cards.len() as u64;
            let offset = ((page.saturating_sub(1)) as usize) * ENTITY_INDEX_PAGE_SIZE as usize;
            let slice: Vec<EntityIndexCard> = cards
                .into_iter()
                .skip(offset)
                .take(ENTITY_INDEX_PAGE_SIZE as usize)
                .collect();
            Ok((slice, total))
        }
        Err(e) => Err(e),
    }
}

fn entity_list_sort_from_sort_key(sort: &SortKey) -> EntityListSort {
    match sort {
        SortKey::Entity(EntitySortKey::VideoCount) => EntityListSort::VideoCountDesc,
        SortKey::Entity(EntitySortKey::Alphabetical) => EntityListSort::NameAsc,
        SortKey::Entity(EntitySortKey::Trending) | SortKey::None => EntityListSort::Trending,
        SortKey::Video(_) | SortKey::Search(_) => EntityListSort::Trending,
    }
}

fn sort_fixture_pornstars(cards: &mut [EntityIndexCard], sort: &SortKey) {
    match sort {
        SortKey::Entity(EntitySortKey::VideoCount) => {
            cards.sort_by(|a, b| {
                b.video_count
                    .cmp(&a.video_count)
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        SortKey::Entity(EntitySortKey::Alphabetical) => {
            cards.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        }
        _ => {
            cards.sort_by(|a, b| {
                b.video_count
                    .cmp(&a.video_count)
                    .then_with(|| a.display_name.cmp(&b.display_name))
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

pub async fn channels_list(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    channels_index_response(pool, cfg, None, query).await
}

pub async fn channels_list_page(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    channels_index_response(pool, cfg, Some(path.into_inner()), query).await
}

async fn channels_index_response(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let listing = ListingKind::EntityIndex(EntityIndexKind::Channels);
    let path_page_str = path_page.map(|p| p.to_string());
    let path_page_ref = path_page_str.as_deref();

    let (cards, total_items) = load_channels_page(pool.get_ref(), &query, path_page_ref).await?;
    let page = crate::models::pagination::resolve_page(path_page_ref, &query)
        .map_err(pagination_to_app_error)?;
    let spec = crate::models::pagination::build_page_spec(
        listing.clone(),
        page,
        total_items,
        &query,
        Some(ENTITY_INDEX_PAGE_SIZE),
    )
    .map_err(pagination_to_app_error)?;
    let meta =
        crate::models::pagination::pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::channels_index(layout.clone(), &meta);
    let channels = ChannelsIndexView::build(
        cards,
        &meta,
        &query.parse_sort_for_listing(&ListingKind::EntityIndex(EntityIndexKind::Channels)),
        &layout.media_cdn,
    );

    let html = ChannelsTemplate { ctx, channels }
        .render()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "channels"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn load_channels_page(
    pool: &DbPool,
    query: &ListingQueryParams,
    path_page: Option<&str>,
) -> Result<(Vec<EntityIndexCard>, u64), AppError> {
    let listing = ListingKind::EntityIndex(EntityIndexKind::Channels);
    let sort_key = query.parse_sort_for_listing(&listing);
    let entity_sort = entity_list_sort_from_sort_key(&sort_key);
    let page = crate::models::pagination::resolve_page(path_page, query)
        .map_err(pagination_to_app_error)?;

    match list_channels_index(pool, entity_sort, page, ENTITY_INDEX_PAGE_SIZE).await {
        Ok(page_data) if !page_data.items.is_empty() => Ok((page_data.items, page_data.total)),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            let mut cards = seed_channel_cards(&seed);
            sort_fixture_channels(&mut cards, &sort_key);
            let total = cards.len() as u64;
            let offset = ((page.saturating_sub(1)) as usize) * ENTITY_INDEX_PAGE_SIZE as usize;
            let slice: Vec<EntityIndexCard> = cards
                .into_iter()
                .skip(offset)
                .take(ENTITY_INDEX_PAGE_SIZE as usize)
                .collect();
            Ok((slice, total))
        }
        Err(e) => Err(e),
    }
}

fn sort_fixture_channels(cards: &mut [EntityIndexCard], sort: &SortKey) {
    sort_fixture_pornstars(cards, sort);
}

pub async fn tags_hub(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let tags = load_tags_hub(pool.get_ref()).await?;
    let ctx = RenderContext::tags_hub(layout);
    let html = TagsTemplate {
        ctx,
        tags: TagsHubView::build(&tags),
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "tags"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

async fn load_tags_hub(pool: &DbPool) -> Result<Vec<TagRow>, AppError> {
    match list_tags_for_index(pool).await {
        Ok(tags) => Ok(tags),
        Err(AppError::Db(_)) => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

pub async fn category_slug(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<String>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    let slug = path.into_inner();
    slug_listing_response(pool, cfg, &slug, None, query).await
}

pub async fn category_slug_page(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    path: web::Path<(String, u32)>,
    query: web::Query<ListingQueryParams>,
) -> Result<impl Responder, AppError> {
    let (slug, page) = path.into_inner();
    slug_listing_response(pool, cfg, &slug, Some(page), query).await
}

async fn slug_listing_response(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
    slug: &str,
    path_page: Option<u32>,
    query: web::Query<ListingQueryParams>,
) -> Result<HttpResponse, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());

    let resolved = resolve_listing_target(pool.get_ref(), slug).await?;
    let Some(target) = resolved else {
        return Ok(not_found_page_response(slug, "category_slug"));
    };

    let listing = ListingKind::CategorySlug {
        slug: slug.to_string(),
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
    let video_sort = listing_sort.to_video_list_sort();

    let (videos, total_items) =
        load_slug_videos(pool.get_ref(), &target, slug, page, video_sort, hd_only).await?;

    let spec = build_page_spec(
        listing.clone(),
        page,
        total_items,
        &query,
        Some(DEFAULT_HOME_VIDEO_PER_PAGE),
    )
    .map_err(pagination_to_app_error)?;
    let meta = pagination_meta(&spec, total_items, &layout.site_base_url);

    let ctx = RenderContext::slug_listing(layout.clone(), &target.header, &meta);
    let page_view = SlugListingView::build(&target.header, videos, &meta, &sort_key, hd, listing);

    let html = SlugListingTemplate {
        ctx,
        page: page_view,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "category_slug"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

struct SlugListingTarget {
    kind: ListingSlugKind,
    id: u64,
    header: TaxonomyListingHeader,
}

async fn resolve_listing_target(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<SlugListingTarget>, AppError> {
    match resolve_listing_target_db(pool, slug).await {
        Ok(Some(t)) => Ok(Some(t)),
        Ok(None) => Ok(resolve_listing_target_fixture(slug)),
        Err(AppError::Db(_)) => Ok(resolve_listing_target_fixture(slug)),
        Err(e) => Err(e),
    }
}

async fn resolve_listing_target_db(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<SlugListingTarget>, AppError> {
    if let Some(cat) = get_category_by_slug(pool, slug).await? {
        return Ok(Some(SlugListingTarget {
            kind: ListingSlugKind::Category,
            id: cat.id,
            header: cat.to_listing_header(),
        }));
    }
    if let Some(tag) = get_tag_by_slug(pool, slug).await? {
        return Ok(Some(SlugListingTarget {
            kind: ListingSlugKind::Tag,
            id: tag.id,
            header: tag.to_listing_header(),
        }));
    }
    Ok(None)
}

fn resolve_listing_target_fixture(slug: &str) -> Option<SlugListingTarget> {
    let seed = load_catalog_seed().ok()?;
    seed.categories.iter().find(|c| c.slug == slug).map(|c| {
        let row = CategoryRow {
            id: 0,
            slug: c.slug.clone(),
            display_name: c.display_name.clone(),
            description: None,
            thumb_url: Some(c.thumb_url.clone()),
            video_count: c.video_count,
            intro_html: None,
            sort_order: c.sort_order as i32,
            is_active: true,
        };
        SlugListingTarget {
            kind: ListingSlugKind::Category,
            id: 0,
            header: row.to_listing_header(),
        }
    })
}

async fn load_slug_videos(
    pool: &DbPool,
    target: &SlugListingTarget,
    slug: &str,
    page: u32,
    sort: crate::models::video::VideoListSort,
    hd_only: bool,
) -> Result<(Vec<VideoThumb>, u64), AppError> {
    let per_page = DEFAULT_HOME_VIDEO_PER_PAGE;

    // DB path (when we have a real id).
    if target.id != 0 {
        let db_result = match target.kind {
            ListingSlugKind::Category => {
                let videos =
                    list_thumbs_for_category(pool, target.id, page, per_page, sort, hd_only).await;
                let total = count_videos_for_category(pool, target.id, hd_only).await;
                (videos, total)
            }
            ListingSlugKind::Tag => {
                let videos =
                    list_thumbs_for_tag(pool, target.id, page, per_page, sort, hd_only).await;
                let total = count_videos_for_tag(pool, target.id, hd_only).await;
                (videos, total)
            }
        };
        if let (Ok(videos), Ok(total)) = db_result {
            if total > 0 {
                return Ok((videos, total));
            }
        }
    }

    // Fixture fallback (category links only; tags have no fixture links yet).
    let seed = load_catalog_seed()?;
    let mut all = seed_thumbs_for_category(&seed, slug);
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

fn sort_fixture_thumbs(thumbs: &mut [VideoThumb], sort: crate::models::video::VideoListSort) {
    use crate::models::video::VideoListSort;
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
