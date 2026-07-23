use actix_web::{web, HttpResponse, Responder};
use askama::Template;

use crate::config::Config;
use crate::db::DbPool;
use crate::errors::{not_found_page_response, AppError};
use crate::fixtures::{
    live_channel_index_cards, live_pornstar_index_cards, load_catalog_seed, seed_channel_cards,
    seed_pornstar_cards, seed_thumbs_for_category,
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

const LIVE_PORNSTAR_INDEX_TOTAL_PAGES: u64 = 83;
const LIVE_CHANNEL_INDEX_TOTAL_PAGES: u64 = 31;

pub async fn categories(
    pool: web::Data<DbPool>,
    cfg: web::Data<Config>,
) -> Result<impl Responder, AppError> {
    let layout = SiteLayout::from_config(cfg.get_ref());
    let page = load_categories_page_data(pool.get_ref(), &layout).await?;
    let ctx = RenderContext::categories_index(layout);
    let html = CategoriesTemplate {
        ctx,
        categories: page.categories.clone(),
        top_tags: page.top_tags.clone(),
        tag_preload_slugs: page.tag_preload_slugs.clone(),
        top_pornstars: page.top_pornstars.clone(),
    }
    .render()
    .unwrap();
    let html = normalize_live_shell_html(normalize_categories_live_sections(html, &page));
    Ok(HttpResponse::Ok()
        .insert_header((HANDLER_MARKER, "categories"))
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn normalize_categories_live_sections(
    html: String,
    page: &crate::views::CategoriesPageData,
) -> String {
    let html = replace_html_section(
        html,
        r#"<div class="all_cats">"#,
        r#"<div id="ajax_content"></div>"#,
        &format!(
            r#"<div class="all_cats"> {} </div> "#,
            render_live_category_grid(&page.categories)
        ),
    );
    let html = replace_html_section(
        html,
        r#"<div class="tags-list" style="text-transform: uppercase;">"#,
        r#"<div class="sect-title" style="margin-top:20px">"#,
        &format!(
            r#"<div class="tags-list" style="text-transform: uppercase;"> {} </div> "#,
            render_live_top_tags(&page.top_tags)
        ),
    );
    replace_html_section(
        html,
        r#"<div class="all_pornstars">"#,
        r#"<!-- end <div class="all_pornstars"> -->"#,
        &format!(
            r#"<div class="all_pornstars"> {} </div> "#,
            render_live_top_pornstars(&page.top_pornstars)
        ),
    )
}

fn render_live_top_pornstars(pornstars: &[crate::views::CategoriesTopPornstar]) -> String {
    pornstars
        .iter()
        .map(|ps| {
            let href = live_escape(&ps.profile_url);
            let thumb = live_escape(&ps.thumb_url);
            let name = live_escape(&ps.display_name);
            format!(
                concat!(
                    r#"<div class="thumb cat" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
                    r#"<a href="{href}" class="thumb-in" itemprop="url"> "#,
                    r#"<div class="thumb-img"> "#,
                    r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" itemprop="contentUrl" alt="{name}" />"#,
                    r#"<noscript><img class="thumb-cover" src="{thumb}" itemprop="contentUrl" alt="{name}" /></noscript> "#,
                    r##"<span class="count-videos"><svg fill="#fff"><use xlink:href="#camera-svg"></use></svg>{count}</span> "##,
                    r#"</div> <div class="thumb-title" itemprop="name">{name}</div> </a> </div>"#
                ),
                href = href,
                lazy = LIVE_LAZY_GIF,
                thumb = thumb,
                name = name,
                count = ps.video_count,
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_live_unpaginated_head_spacing(html: String) -> String {
    let html = html.replace(
        r#"<link rel="canonical" href="https://pornsok.com/categories" /><link rel="dns-prefetch""#,
        r#"<link rel="canonical" href="https://pornsok.com/categories" /> <link rel="dns-prefetch""#,
    );
    html.replace(
        r#"<link rel="canonical" href="https://pornsok.com/" /><link rel="dns-prefetch" href="//c.foxporn.tv"> <link rel="preconnect" href="//c.foxporn.tv"> <meta name="rating""#,
        r#"<link rel="canonical" href="https://pornsok.com/" /> <link rel="dns-prefetch" href="//c.foxporn.tv"> <link rel="preconnect" href="//c.foxporn.tv"> <link rel="next" href="https://pornsok.com/2"> <meta name="rating""#,
    )
}

fn normalize_live_static_cdn_paths(html: String) -> String {
    let html = html.replace(
        r#"background:url("https:/fox-tpl/images/shadow.png")"#,
        r#"background:url("https://c.foxporn.tv/fox-tpl/images/shadow.png")"#,
    );
    let html = html.replace(
        "https://c.foxporn.tv/static/fox-tpl/",
        "https://c.foxporn.tv/fox-tpl/",
    );
    let html = html.replace("/static/fox-tpl/", "/fox-tpl/");
    let html = html.replace(
        r#"src="/static/js/main.min.js""#,
        r#"src="/fox-tpl/js/main.min.js?v=11""#,
    );
    html.replace(
        r#"src="/static/js/rocket-loader.min.js""#,
        r#"src="/fox-tpl/js/rocket-loader.min.js""#,
    )
}

fn normalize_live_boot_script_spacing(html: String) -> String {
    html.replace(
        r#">  var isTHUMBS_OR_PLAYER"#,
        r#"> var isTHUMBS_OR_PLAYER"#,
    )
    .replace(r#"});  </script>"#, r#"}); </script>"#)
}

fn replace_html_section(html: String, start: &str, end: &str, replacement: &str) -> String {
    let Some(start_idx) = html.find(start) else {
        return html;
    };
    let body_start = start_idx + start.len();
    let Some(end_offset) = html[body_start..].find(end) else {
        return html;
    };
    let end_idx = body_start + end_offset;
    let mut out = String::with_capacity(html.len() + replacement.len());
    out.push_str(&html[..start_idx]);
    out.push_str(replacement);
    out.push_str(&html[end_idx..]);
    out
}

fn render_live_category_grid(categories: &[crate::models::taxonomy::CategoryCard]) -> String {
    categories
        .iter()
        .map(render_live_category_card)
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_live_category_card(card: &crate::models::taxonomy::CategoryCard) -> String {
    let href = live_escape(&card.listing_url);
    let thumb = live_escape(&card.thumb_url);
    let title = live_escape(&card.title);
    let alt = live_escape(&card.alt_text);
    let link_title = card
        .link_title
        .as_ref()
        .map(|title| format!(r#" title="{}""#, live_escape(title)))
        .unwrap_or_default();
    let image = if card.lazy {
        let noscript = if card.listing_url == "/?hd=1" || card.listing_url == "/tags" {
            format!(
                r#"<noscript><img class="thumb-cover" src="{thumb}" alt="{alt}" itemprop="contentUrl" /></noscript> "#,
                thumb = thumb,
                alt = alt,
            )
        } else {
            format!(
                r#"<noscript><img class="thumb-cover" src="{thumb}" itemprop="contentUrl" /></noscript>"#,
                thumb = thumb,
            )
        };
        format!(
            r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" alt="{alt}" />{noscript}"#,
            lazy = LIVE_LAZY_GIF,
            thumb = thumb,
            alt = alt,
            noscript = noscript,
        )
    } else {
        format!(
            r#"<img class="thumb-cover" src="{thumb}" itemprop="contentUrl" alt="{alt}" />"#,
            thumb = thumb,
            alt = alt,
        )
    };
    let count = if card.uses_tags_icon {
        format!(
            r#"<span class="count-videos"><i class="fa fa-tags"></i><span class="count-tags">{}</span></span>"#,
            card.video_count
        )
    } else {
        format!(
            r##"<span class="count-videos"><svg fill="#fff"><use xlink:href="#camera-svg" /></svg>{}</span>"##,
            card.video_count
        )
    };
    format!(
        concat!(
            r#"<div class="thumb cat" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
            r#"<a href="{href}"{link_title} class="thumb-in" itemprop="url"> "#,
            r#"<div class="thumb-img"> {image}{count} </div> "#,
            r#"<div class="thumb-title" itemprop="name">{title}</div> </a> </div>"#
        ),
        href = href,
        link_title = link_title,
        image = image,
        count = count,
        title = title,
    )
}

fn render_live_top_tags(tags: &[crate::views::CategoriesTopTag]) -> String {
    let items = tags
        .iter()
        .map(|tag| {
            format!(
                concat!(
                    r#"<li><i class="fa fa-tag"></i><a href="{href}" "#,
                    r#"onMouseMove="if (!window.__cfRLUnblockHandlers) return false; ShowVisualBox(event, '{mini}', false, false, {count}, 150, '{hover}')" "#,
                    r#"onMouseOut="if (!window.__cfRLUnblockHandlers) return false; HideVisualBox()" "#,
                    r#"data-cf-modified-ee28ed6a23599dc985b82107-="">{label}</a></li>"#
                ),
                href = live_escape(&tag.listing_url),
                mini = live_escape(&tag.mini_url),
                count = tag.hover_count,
                hover = live_escape(&tag.hover_label),
                label = live_escape(&tag.label),
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let preload = tags
        .iter()
        .map(|tag| format!("'{}'", live_escape(&tag.slug)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            r#"<ul> {items} </ul> "#,
            r#"<script type="text/javascript">preloads.push({{'before': 'https://c.foxporn.tv/fox-images/categories/', 'after': '-mini.jpg', 'array': [{preload}], 'on_scroll': true, 'delay': 2}});</script>"#
        ),
        items = items,
        preload = preload,
    )
}

const LIVE_LAZY_GIF: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

fn live_escape(raw: &str) -> String {
    raw.replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    let html = normalize_live_shell_html(html);

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
        Ok(page_data) if !page_data.items.is_empty() => {
            if entity_page_is_underfilled(page_data.items.len(), page_data.total) {
                if let Ok(fixture_page) = load_pornstars_fixture_page(&sort_key, page) {
                    if fixture_page.1 > page_data.total {
                        return Ok(fixture_page);
                    }
                }
            }
            Ok((page_data.items, page_data.total))
        }
        Ok(_) | Err(AppError::Db(_)) => load_pornstars_fixture_page(&sort_key, page),
        Err(e) => Err(e),
    }
}

fn entity_page_is_underfilled(item_count: usize, total: u64) -> bool {
    item_count < ENTITY_INDEX_PAGE_SIZE as usize || total < u64::from(ENTITY_INDEX_PAGE_SIZE)
}

fn load_pornstars_fixture_page(
    sort_key: &SortKey,
    page: u32,
) -> Result<(Vec<EntityIndexCard>, u64), AppError> {
    let mut cards = live_pornstar_index_cards().unwrap_or_default();
    let preserve_source_order = !cards.is_empty();
    if cards.is_empty() {
        let seed = load_catalog_seed()?;
        cards = seed_pornstar_cards(&seed);
    }
    sort_fixture_entity_cards(&mut cards, sort_key, preserve_source_order);
    Ok(slice_entity_fixture_page(
        cards,
        page,
        live_entity_total_items(LIVE_PORNSTAR_INDEX_TOTAL_PAGES),
    ))
}

fn entity_list_sort_from_sort_key(sort: &SortKey) -> EntityListSort {
    match sort {
        SortKey::Entity(EntitySortKey::VideoCount) => EntityListSort::VideoCountDesc,
        SortKey::Entity(EntitySortKey::Alphabetical) => EntityListSort::NameAsc,
        SortKey::Entity(EntitySortKey::Trending) | SortKey::None => EntityListSort::Trending,
        SortKey::Video(_) | SortKey::Search(_) => EntityListSort::Trending,
    }
}

fn sort_fixture_entity_cards(
    cards: &mut [EntityIndexCard],
    sort: &SortKey,
    preserve_default_order: bool,
) {
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
        _ if !preserve_default_order => {
            cards.sort_by(|a, b| {
                b.video_count
                    .cmp(&a.video_count)
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }
        _ => {}
    }
}

fn slice_entity_fixture_page(
    cards: Vec<EntityIndexCard>,
    page: u32,
    total_items: u64,
) -> (Vec<EntityIndexCard>, u64) {
    let offset = ((page.saturating_sub(1)) as usize) * ENTITY_INDEX_PAGE_SIZE as usize;
    let slice: Vec<EntityIndexCard> = cards
        .into_iter()
        .skip(offset)
        .take(ENTITY_INDEX_PAGE_SIZE as usize)
        .collect();
    (slice, total_items)
}

fn live_entity_total_items(total_pages: u64) -> u64 {
    total_pages * u64::from(ENTITY_INDEX_PAGE_SIZE)
}

pub(super) fn normalize_live_shell_html(html: String) -> String {
    let html = html.strip_prefix('\n').unwrap_or(&html).to_string();
    let html = html.replacen(
        r#"<html lang="en" prefix="og: http://ogp.me/ns#" data-theme="dark">"#,
        r#"<html lang="en" prefix="og: http://ogp.me/ns#" >"#,
        1,
    );
    let html = normalize_live_unpaginated_head_spacing(html);
    let html = normalize_live_static_cdn_paths(html);
    let html = normalize_live_boot_script_spacing(html);
    let html = normalize_live_head_link_order(html);
    let html = normalize_live_theme_state(html);
    let html = normalize_live_channel_svg_symbol(html);
    let html = normalize_live_pornstar_menu_tail(html);
    let html = normalize_live_channel_menu_tail(html);
    insert_live_menu_preloads(html)
}

fn normalize_live_head_link_order(html: String) -> String {
    const RESOURCE_HINTS: &str = r#"<link rel="dns-prefetch" href="//c.foxporn.tv"> <link rel="preconnect" href="//c.foxporn.tv">"#;
    let Some(resource_start) = html.find(RESOURCE_HINTS) else {
        return html;
    };
    let Some(next_start) = html[..resource_start].rfind(r#"<link rel="next" href=""#) else {
        return html;
    };
    let Some(next_end_offset) = html[next_start..].find(r#"" />"#) else {
        return html;
    };
    let next_end = next_start + next_end_offset + r#"" />"#.len();
    let next_href =
        &html[next_start + r#"<link rel="next" href=""#.len()..next_end - r#"" />"#.len()];

    let mut out = String::with_capacity(html.len());
    out.push_str(&html[..next_start]);
    out.push(' ');
    out.push_str(RESOURCE_HINTS);
    out.push_str(r#" <link rel="next" href=""#);
    out.push_str(next_href);
    out.push_str(r#"">"#);
    out.push_str(&html[resource_start + RESOURCE_HINTS.len()..]);
    out
}

fn normalize_live_theme_state(html: String) -> String {
    html.replace(r#"<body class="black">"#, "<body>")
        .replace(
        r#"id="inner-vk-svg" style="fill:#090909""#,
        r#"id="inner-vk-svg" style="fill:#fefefe""#,
    )
    .replace(
        r#"id="telegram-vk-svg" style="fill:#090909""#,
        r#"id="telegram-vk-svg" style="fill:#fefefe""#,
    )
    .replace(
        r##"<div id="day-night" title="Day mode"> <svg id="day-night-icon" class="to-day"><use xlink:href="#sun-svg" /></svg>"##,
        r##"<div id="day-night" title="Night mode"> <svg id="day-night-icon" class="to-night"><use xlink:href="#moon-svg" /></svg>"##,
    )
}

fn normalize_live_channel_svg_symbol(html: String) -> String {
    if !html.contains(r##"<use xlink:href="#tv-svg" />"##) {
        return html;
    }
    const LOCAL_STAR_SYMBOL: &str = r#"<symbol id="star-svg" viewBox="0 0 576 512"><path d="M528.1 171.5L382 150.2 316.7 17.8c-11.7-23.6-45.6-23.9-57.4 0L194 150.2 47.9 171.5c-26.2 3.8-36.7 36.1-17.7 54.6l105.7 103-25 145.5c-4.5 26.3 23.2 46 46.4 33.7L288 439.6l130.7 68.7c23.2 12.2 50.9-7.4 46.4-33.7l-25-145.5 105.7-103c19-18.5 8.5-50.8-17.7-54.6zM388.6 312.3l23.7 138.4L288 385.4l-124.3 65.3 23.7-138.4-100.6-98 139-20.2 62.2-126 62.2 126 139 20.2-100.6 98z"/></symbol>"#;
    const LIVE_TV_SYMBOL: &str = r#"<symbol id="tv-svg" viewBox="0 0 512 512"><path d="M490.594 144.054c-44.404-6.384-91.691-11.008-141.038-13.61l82.444-82.444-32-32-112.279 112.278c-10.503-0.184-21.078-0.278-31.721-0.278v0l-128-128-32 32 97.098 97.098c-60.461 2.121-118.169 7.262-171.693 14.956-13.766 53.863-21.405 113.376-21.405 175.946s7.639 122.083 21.402 175.945c71.821 10.326 151.17 16.055 234.598 16.055s162.775-5.729 234.594-16.055c13.767-53.862 21.406-113.375 21.406-175.945s-7.639-122.083-21.406-175.946zM431.946 437.297c-53.865 6.883-113.375 10.703-175.946 10.703s-122.083-3.82-175.946-10.703c-10.324-35.908-16.054-75.583-16.054-117.297s5.729-81.39 16.054-117.298c53.863-6.883 113.375-10.702 175.946-10.702 62.568 0 122.081 3.819 175.943 10.702 10.328 35.908 16.057 75.583 16.057 117.298 0 41.714-5.729 81.389-16.054 117.297z"/></symbol>"#;
    html.replacen(LOCAL_STAR_SYMBOL, LIVE_TV_SYMBOL, 1).replace(
        r#"<div id="show_sort" class="button">Sort by:<span class="arrow"></span></div>"#,
        r#"<button id="show_sort" class="mobile-sort">Sort by:<span class="arrow"></span></button>"#,
    )
}

fn normalize_live_pornstar_menu_tail(html: String) -> String {
    const START_MARKER: &str =
        r#"<a class="sub-url" data-icon="model" href="/pornstar/thestartofus">"#;
    const END_MARKER: &str = r#"<li> <a class="all-url" href="/pornstars">"#;

    let Some(start) = html.find(START_MARKER) else {
        return html;
    };
    let Some(end_offset) = html[start..].find(END_MARKER) else {
        return html;
    };
    let end = start + end_offset;
    let mut out = String::with_capacity(html.len() + 18);
    out.push_str(&html[..start]);
    out.push_str(&live_pornstar_menu_tail_html());
    out.push_str(&html[end..]);
    out
}

fn live_pornstar_menu_tail_html() -> String {
    const LAZY: &str =
        "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";
    const ITEMS: &[(&str, &str, &str, &str)] = &[
        ("pkgulnaz", "pkgulnaz", "Pkgulnaz", "Pkgulnaz"),
        ("yoursoniya", "yoursoniya", "Yoursoniya", "Yoursoniya"),
        ("riley-reid", "riley-reid", "Riley Reid", "Riley Reid"),
        ("gina-gerson", "gina-gerson", "Gina Gerson", "Gina Gerson"),
        (
            "kelly-aleman",
            "kelly-aleman",
            "Kelly Aleman",
            "Kelly Aleman",
        ),
        ("anny-walker", "anny-walker", "Anny Walker", "Anny Walker"),
        (
            "tumanovaalina",
            "tumanovaalina",
            "Alina Tumanova",
            "Alina Tumanova",
        ),
        (
            "whitney-wright",
            "whitney-wright",
            "Whitney Wright",
            "Whitney Wright",
        ),
        (
            "sheila-ortega",
            "sheila-ortega",
            "Sheila Ortega",
            "Sheila Ortega",
        ),
        (
            "abella-danger",
            "abella-danger",
            "Abella Danger",
            "Abella Danger",
        ),
    ];

    let mut out = String::new();
    for (idx, (slug, img_slug, alt, label)) in ITEMS.iter().enumerate() {
        if idx > 0 {
            out.push_str("<li> ");
        }
        out.push_str(&format!(
            concat!(
                r#"<a class="sub-url" data-icon="model" href="/pornstar/{slug}"> "#,
                r#"<div class="menu-thumb-holder"> "#,
                r#"<img class="menu-pic" src="{lazy}" data-mobile="/fox-tpl/images/spacer.gif" "#,
                r#"data-mini="https://c.foxporn.tv/fox-images/pornstars/m-{img_slug}.jpg" alt="{alt}" /> "#,
                r#"</div> <span class="menu-label">{label}</span> </a> </li> "#
            ),
            slug = slug,
            lazy = LAZY,
            img_slug = img_slug,
            alt = alt,
            label = label,
        ));
    }
    out
}

fn normalize_live_channel_menu_tail(html: String) -> String {
    const START_MARKER: &str =
        r#"<a class="sub-url" data-icon="tv" href="/channel/defloration-tv">"#;
    const END_MARKER: &str = r#"<li> <a class="all-url" href="/channels">"#;

    let Some(start) = html.find(START_MARKER) else {
        return html;
    };
    let Some(end_offset) = html[start..].find(END_MARKER) else {
        return html;
    };
    let end = start + end_offset;
    let mut out = String::with_capacity(html.len());
    out.push_str(&html[..start]);
    out.push_str(&live_channel_menu_tail_html());
    out.push_str(&html[end..]);
    out
}

fn live_channel_menu_tail_html() -> String {
    const LAZY: &str =
        "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";
    const ITEMS: &[(&str, &str, &str)] = &[
        ("brazzers", "Brazzers", "Brazzers"),
        ("oldje", "Oldje", "Oldje"),
        ("blacked", "Blacked", "Blacked"),
        ("mofos", "Mofos", "Mofos"),
        ("teen-erotica", "Teen Erotica", "Teen Erotica"),
        (
            "bang-bros-network",
            "Bang Bros Network",
            "Bang Bros Network",
        ),
        ("dogfart-network", "Dogfart Network", "Dogfart Network"),
        ("wow-girls", "Wow Girls", "Wow Girls"),
        (
            "my-friends-hot-mom",
            "My Friends Hot Mom",
            "My Friends Hot Mom",
        ),
        ("bratty-sis", "Bratty Sis", "Bratty Sis"),
        ("blacked-raw", "Blacked Raw", "Blacked Raw"),
        ("letsdoeit", "LetsDoeIt", "LetsDoeIt"),
        ("reality-kings", "Reality Kings", "Reality Kings"),
        ("mom-xxx", "Mom XXX", "Mom XXX"),
        (
            "pornstar-platinum",
            "Pornstar Platinum",
            "Pornstar Platinum",
        ),
        ("mommys-girl", "Mommys Girl", "Mommys Girl"),
        ("vixen", "Vixen", "Vixen"),
    ];

    let mut out = String::new();
    for (idx, (slug, alt, label)) in ITEMS.iter().enumerate() {
        if idx > 0 {
            out.push_str("<li> ");
        }
        out.push_str(&format!(
            concat!(
                r#"<a class="sub-url" data-icon="tv" href="/channel/{slug}"> "#,
                r#"<div class="menu-thumb-holder"> "#,
                r#"<img class="menu-pic" src="{lazy}" data-mobile="/fox-tpl/images/spacer.gif" "#,
                r#"data-mini="https://c.foxporn.tv/fox-images/channels/{slug}.jpg" alt="{alt}" /> "#,
                r#"</div> <span class="menu-label">{label}</span> </a> </li> "#
            ),
            slug = slug,
            lazy = LAZY,
            alt = alt,
            label = label,
        ));
    }
    out
}

fn insert_live_menu_preloads(html: String) -> String {
    const TOKEN: &str = "ee28ed6a23599dc985b82107";
    const INSERT_MARKER: &str =
        r#"</nav> <div class="search-box" itemscope="" itemtype="http://schema.org/WebSite">"#;
    if html.contains("prel_top_cats") {
        return html;
    }
    let preload_script = format!(
        concat!(
            r#"</nav> <script type="{token}-text/javascript"> "#,
            r#"preloads.push({{'before': 'https://c.foxporn.tv/fox-images/categories/', 'after': '-mini.jpg', 'array': ["lesbian","blowjob","milf","cumshot","creampie","ebony","anal","pawg","big-dick","bbw","eating-pussy","hairy-pussy","deep-throat","mom-and-son","femdom","teen","latina"], 'delay': 2, 'cookie': 'prel_top_cats', 'cookie_days': 14}}); "#,
            r#"preloads.push({{'before': 'https://c.foxporn.tv/fox-images/pornstars/', 'array': ["m-your-priya.jpg","m-sapphire-lapiedra.jpg","m-yourxdarling.jpg","m-garmia.jpg","m-savannah-watson.jpg","m-chloe-cherry.jpg","m-athena-faris.jpg","m-pkgulnaz.jpg","m-yoursoniya.jpg","m-riley-reid.jpg","m-gina-gerson.jpg","m-kelly-aleman.jpg","m-anny-walker.jpg","m-tumanovaalina.jpg","m-whitney-wright.jpg","m-sheila-ortega.jpg","m-abella-danger.jpg"], 'delay': 2, 'cookie': 'prel_top_pornsars', 'cookie_days': 14}}); "#,
            r#"preloads.push({{'before': 'https://c.foxporn.tv/fox-images/channels/', 'array': ["brazzers.jpg","oldje.jpg","blacked.jpg","mofos.jpg","teen-erotica.jpg","bang-bros-network.jpg","dogfart-network.jpg","wow-girls.jpg","my-friends-hot-mom.jpg","bratty-sis.jpg","blacked-raw.jpg","letsdoeit.jpg","reality-kings.jpg","mom-xxx.jpg","pornstar-platinum.jpg","mommys-girl.jpg","vixen.jpg"], 'delay': 2, 'cookie': 'prel_top_channels', 'cookie_days': 14}}); "#,
            r#"</script> <div class="search-box" itemscope="" itemtype="http://schema.org/WebSite">"#
        ),
        token = TOKEN,
    );
    html.replacen(INSERT_MARKER, &preload_script, 1)
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
    let sort_key =
        query.parse_sort_for_listing(&ListingKind::EntityIndex(EntityIndexKind::Channels));
    let preload_cards = if meta.has_next {
        load_channels_preload_cards(pool.get_ref(), &sort_key, page + 1).await
    } else {
        Vec::new()
    };
    let channels = ChannelsIndexView::build_with_preload_cards(
        cards,
        preload_cards,
        &meta,
        &sort_key,
        &layout.media_cdn,
    );

    let html = ChannelsTemplate { ctx, channels }
        .render()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let html = normalize_live_shell_html(html);

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
        Ok(page_data) if !page_data.items.is_empty() => {
            if entity_page_is_underfilled(page_data.items.len(), page_data.total) {
                if let Ok(fixture_page) = load_channels_fixture_page(&sort_key, page) {
                    if fixture_page.1 > page_data.total {
                        return Ok(fixture_page);
                    }
                }
            }
            Ok((page_data.items, page_data.total))
        }
        Ok(_) | Err(AppError::Db(_)) => load_channels_fixture_page(&sort_key, page),
        Err(e) => Err(e),
    }
}

fn load_channels_fixture_page(
    sort_key: &SortKey,
    page: u32,
) -> Result<(Vec<EntityIndexCard>, u64), AppError> {
    let mut cards = live_channel_index_cards().unwrap_or_default();
    let preserve_source_order = !cards.is_empty();
    if cards.is_empty() {
        let seed = load_catalog_seed()?;
        cards = seed_channel_cards(&seed);
    }
    sort_fixture_entity_cards(&mut cards, sort_key, preserve_source_order);
    Ok(slice_entity_fixture_page(
        cards,
        page,
        live_entity_total_items(LIVE_CHANNEL_INDEX_TOTAL_PAGES),
    ))
}

async fn load_channels_preload_cards(
    pool: &DbPool,
    sort_key: &SortKey,
    page: u32,
) -> Vec<EntityIndexCard> {
    let entity_sort = entity_list_sort_from_sort_key(sort_key);
    match list_channels_index(pool, entity_sort, page, ENTITY_INDEX_PAGE_SIZE).await {
        Ok(page_data) if !page_data.items.is_empty() => page_data.items,
        _ => load_channels_fixture_page(sort_key, page)
            .map(|(cards, _)| cards)
            .unwrap_or_default(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_pornstar_fixture_page_uses_first_page_size_and_source_order() {
        let sort = SortKey::Entity(EntitySortKey::Trending);
        let (cards, total) = load_pornstars_fixture_page(&sort, 1).expect("pornstar fixture page");

        assert_eq!(cards.len(), ENTITY_INDEX_PAGE_SIZE as usize);
        assert_eq!(
            total,
            LIVE_PORNSTAR_INDEX_TOTAL_PAGES * u64::from(ENTITY_INDEX_PAGE_SIZE)
        );
        assert_eq!(cards[0].slug, "your-priya");
    }

    #[test]
    fn live_channel_fixture_page_uses_first_page_size_and_source_order() {
        let sort = SortKey::Entity(EntitySortKey::Trending);
        let (cards, total) = load_channels_fixture_page(&sort, 1).expect("channel fixture page");

        assert_eq!(cards.len(), ENTITY_INDEX_PAGE_SIZE as usize);
        assert_eq!(
            total,
            LIVE_CHANNEL_INDEX_TOTAL_PAGES * u64::from(ENTITY_INDEX_PAGE_SIZE)
        );
        assert_eq!(cards[0].slug, "brazzers");
    }

    #[test]
    fn entity_underfill_detection_matches_live_page_size() {
        assert!(entity_page_is_underfilled(13, 13));
        assert!(entity_page_is_underfilled(
            ENTITY_INDEX_PAGE_SIZE as usize - 1,
            120
        ));
        assert!(!entity_page_is_underfilled(
            ENTITY_INDEX_PAGE_SIZE as usize,
            u64::from(ENTITY_INDEX_PAGE_SIZE)
        ));
    }
}
