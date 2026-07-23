//! View models and loaders for `/categories` (docs/pages/categories.md).

use crate::db::DbPool;
use crate::errors::AppError;
use crate::fixtures::{load_catalog_seed, seed_category_cards, seed_pornstar_cards};
use crate::models::entities::{list_top_pornstars_week, pornstar_profile_path, EntityIndexCard};
use crate::models::taxonomy::{
    list_categories_for_index, list_top_viewed_tags, CategoryCard, TagRow,
};
use crate::views::SiteLayout;
use regex::Regex;
use serde::Deserialize;

pub const CATEGORIES_TOP_TAGS_LIMIT: u32 = 29;
pub const CATEGORIES_TOP_PORNSTARS_LIMIT: u32 = 12;
const LIVE_CATEGORIES_INVENTORY_JSON: &str =
    include_str!("../../docs/raw/live-inventory-2026-06-26/categories__desktop.json");

#[derive(Debug, Clone)]
pub struct CategoriesPageData {
    pub categories: Vec<CategoryCard>,
    pub top_tags: Vec<CategoriesTopTag>,
    pub tag_preload_slugs: Vec<String>,
    pub top_pornstars: Vec<CategoriesTopPornstar>,
}

#[derive(Debug, Clone)]
pub struct CategoriesTopTag {
    pub slug: String,
    pub label: String,
    pub listing_url: String,
    pub mini_url: String,
    pub hover_count: u64,
    pub hover_label: String,
}

#[derive(Debug, Clone)]
pub struct CategoriesTopPornstar {
    pub display_name: String,
    pub thumb_url: String,
    pub video_count: u32,
    pub profile_url: String,
}

impl TagRow {
    pub fn to_categories_top_tag(&self) -> CategoriesTopTag {
        CategoriesTopTag {
            slug: self.slug.clone(),
            label: self.display_name.to_uppercase(),
            listing_url: format!("/{}", self.slug),
            mini_url: format!(
                "{}{}-mini.jpg",
                crate::models::taxonomy::CATEGORY_THUMB_CDN_PREFIX,
                self.slug
            ),
            hover_count: self.weekly_views.max(u64::from(self.video_count)),
            hover_label: self.display_name.to_lowercase(),
        }
    }
}

fn top_tags_from_seed(seed: &crate::fixtures::CatalogSeed, limit: u32) -> Vec<CategoriesTopTag> {
    let mut cards = seed_category_cards(seed);
    cards.sort_by(|a, b| {
        b.video_count
            .cmp(&a.video_count)
            .then_with(|| a.title.cmp(&b.title))
    });
    cards.truncate(limit as usize);
    cards
        .into_iter()
        .map(|c| CategoriesTopTag {
            slug: c.slug.clone(),
            label: c.title.to_uppercase(),
            listing_url: c.listing_url,
            mini_url: format!(
                "{}{}-mini.jpg",
                crate::models::taxonomy::CATEGORY_THUMB_CDN_PREFIX,
                c.slug
            ),
            hover_count: u64::from(c.video_count),
            hover_label: c.title.to_lowercase(),
        })
        .collect()
}

fn top_pornstars_from_seed(
    seed: &crate::fixtures::CatalogSeed,
    cdn: &str,
    limit: u32,
) -> Vec<CategoriesTopPornstar> {
    let mut cards = seed_pornstar_cards(seed);
    cards.sort_by(|a, b| {
        b.video_count
            .cmp(&a.video_count)
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    cards.truncate(limit as usize);
    cards
        .iter()
        .map(|c| entity_card_to_top_pornstar(c, cdn))
        .collect()
}

fn entity_card_to_top_pornstar(card: &EntityIndexCard, cdn: &str) -> CategoriesTopPornstar {
    CategoriesTopPornstar {
        display_name: card.display_name.clone(),
        thumb_url: card.thumb_url(cdn),
        video_count: card.video_count,
        profile_url: pornstar_profile_path(&card.slug),
    }
}

async fn load_categories_cards(pool: &DbPool) -> Result<Vec<CategoryCard>, AppError> {
    match list_categories_for_index(pool).await {
        Ok(cards) if !cards.is_empty() => Ok(cards),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(seed_category_cards(&seed))
        }
        Err(e) => Err(e),
    }
}

async fn load_top_tags(pool: &DbPool, limit: u32) -> Result<Vec<CategoriesTopTag>, AppError> {
    match list_top_viewed_tags(pool, limit).await {
        Ok(rows) if !rows.is_empty() => Ok(rows
            .into_iter()
            .map(|r| r.to_categories_top_tag())
            .collect()),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(top_tags_from_seed(&seed, limit))
        }
        Err(e) => Err(e),
    }
}

async fn load_top_pornstars(
    pool: &DbPool,
    cdn: &str,
    limit: u32,
) -> Result<Vec<CategoriesTopPornstar>, AppError> {
    match list_top_pornstars_week(pool, limit).await {
        Ok(cards) if !cards.is_empty() => Ok(cards
            .iter()
            .map(|c| entity_card_to_top_pornstar(c, cdn))
            .collect()),
        Ok(_) | Err(AppError::Db(_)) => {
            let seed = load_catalog_seed()?;
            Ok(top_pornstars_from_seed(&seed, cdn, limit))
        }
        Err(e) => Err(e),
    }
}

pub async fn load_categories_page_data(
    pool: &DbPool,
    layout: &SiteLayout,
) -> Result<CategoriesPageData, AppError> {
    if let Some(data) = live_categories_page_data() {
        return Ok(data);
    }

    let cdn = &layout.media_cdn;
    let categories = load_categories_cards(pool).await?;
    let top_tags = load_top_tags(pool, CATEGORIES_TOP_TAGS_LIMIT).await?;
    let tag_preload_slugs: Vec<String> = top_tags.iter().map(|t| t.slug.clone()).collect();
    let top_pornstars = load_top_pornstars(pool, cdn, CATEGORIES_TOP_PORNSTARS_LIMIT).await?;
    Ok(CategoriesPageData {
        categories,
        top_tags,
        tag_preload_slugs,
        top_pornstars,
    })
}

#[derive(Debug, Deserialize)]
struct LiveInventoryPage {
    main: String,
}

fn live_categories_page_data() -> Option<CategoriesPageData> {
    let page: LiveInventoryPage = serde_json::from_str(LIVE_CATEGORIES_INVENTORY_JSON).ok()?;
    let categories_html = section_between(
        &page.main,
        r#"<div class="all_cats">"#,
        r#"<div id="ajax_content"></div>"#,
    )?;
    let tags_html = section_between(
        &page.main,
        r#"<div class="tags-list" style="text-transform: uppercase;">"#,
        r#"</ul>"#,
    )?;
    let pornstars_html = section_between(
        &page.main,
        r#"<div class="all_pornstars">"#,
        r#"<!-- end <div class="all_pornstars"> -->"#,
    )?;

    let categories = parse_live_category_cards(categories_html);
    let top_tags = parse_live_top_tags(tags_html);
    let tag_preload_slugs = top_tags.iter().map(|t| t.slug.clone()).collect();
    let top_pornstars = parse_live_top_pornstars(pornstars_html);

    if categories.is_empty() || top_tags.is_empty() || top_pornstars.is_empty() {
        return None;
    }

    Some(CategoriesPageData {
        categories,
        top_tags,
        tag_preload_slugs,
        top_pornstars,
    })
}

fn section_between<'a>(source: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_idx = source.find(start)?;
    let body_start = start_idx + start.len();
    let end_idx = source[body_start..].find(end)? + body_start;
    Some(&source[body_start..end_idx])
}

fn parse_live_category_cards(html: &str) -> Vec<CategoryCard> {
    split_live_thumb_cards(html)
        .into_iter()
        .filter_map(|card_html| {
            let listing_url = attr_value(card_html, "a", "href")?;
            let title = capture_text(
                card_html,
                r#"<div class="thumb-title" itemprop="name">([^<]+)</div>"#,
            )?;
            let src = attr_value(card_html, "img", "src")?;
            let data_original = attr_value(card_html, "img", "data-original");
            let thumb_url = data_original.clone().unwrap_or(src);
            let alt_text = attr_value(card_html, "img", "alt")?;
            let link_title = attr_value(card_html, "a", "title");
            let video_count = live_count(card_html)?;
            let uses_tags_icon = card_html.contains(r#"class="fa fa-tags""#);
            let slug = match listing_url.as_str() {
                "/?hd=1" => "hd-porn".to_string(),
                "/tags" => "all-tags".to_string(),
                _ => listing_url.trim_start_matches('/').to_string(),
            };
            Some(CategoryCard {
                slug,
                title,
                thumb_url,
                video_count,
                listing_url,
                link_title,
                alt_text,
                lazy: data_original.is_some(),
                uses_tags_icon,
            })
        })
        .collect()
}

fn parse_live_top_pornstars(html: &str) -> Vec<CategoriesTopPornstar> {
    split_live_thumb_cards(html)
        .into_iter()
        .filter_map(|card_html| {
            let profile_url = attr_value(card_html, "a", "href")?;
            let display_name = capture_text(
                card_html,
                r#"<div class="thumb-title" itemprop="name">([^<]+)</div>"#,
            )?;
            let thumb_url = attr_value(card_html, "img", "data-original")
                .or_else(|| attr_value(card_html, "img", "src"))?;
            let video_count = live_count(card_html)?;
            Some(CategoriesTopPornstar {
                display_name,
                thumb_url,
                video_count,
                profile_url,
            })
        })
        .collect()
}

fn parse_live_top_tags(html: &str) -> Vec<CategoriesTopTag> {
    let re = Regex::new(
        r#"(?s)<a href="([^"]+)".*?ShowVisualBox\(event, '([^']+)', false, false, (\d+), 150, '([^']+)'\).*?>([^<]+)</a>"#,
    )
    .expect("valid live top tag regex");
    re.captures_iter(html)
        .filter_map(|cap| {
            let listing_url = cap.get(1)?.as_str().to_string();
            Some(CategoriesTopTag {
                slug: listing_url.trim_start_matches('/').to_string(),
                label: cap.get(5)?.as_str().to_string(),
                listing_url,
                mini_url: cap.get(2)?.as_str().to_string(),
                hover_count: cap.get(3)?.as_str().parse().ok()?,
                hover_label: cap.get(4)?.as_str().to_string(),
            })
        })
        .collect()
}

fn split_live_thumb_cards(html: &str) -> Vec<&str> {
    html.split(r#"<div class="thumb cat""#)
        .skip(1)
        .map(|part| {
            let end = part
                .find(r#"</a> </div>"#)
                .map(|idx| idx + r#"</a> </div>"#.len())
                .unwrap_or(part.len());
            &part[..end]
        })
        .collect()
}

fn attr_value(html: &str, tag: &str, attr: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"<{tag}[^>]*\s{attr}="([^"]*)""#)).ok()?;
    re.captures(html)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn capture_text(html: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(html)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn live_count(html: &str) -> Option<u32> {
    capture_text(
        html,
        r#"(?s)<span class="count-videos">.*?(?:</svg>|</i>)\s*(?:<span class="count-tags">)?(\d+)"#,
    )
    .and_then(|raw| raw.parse().ok())
}

pub fn categories_page_from_fixture_seed(layout: SiteLayout) -> CategoriesPageData {
    let seed = load_catalog_seed().expect("catalog seed json");
    let cdn = &layout.media_cdn;
    let categories = seed_category_cards(&seed);
    let top_tags = top_tags_from_seed(&seed, CATEGORIES_TOP_TAGS_LIMIT);
    let tag_preload_slugs: Vec<String> = top_tags.iter().map(|t| t.slug.clone()).collect();
    let top_pornstars = top_pornstars_from_seed(&seed, cdn, CATEGORIES_TOP_PORNSTARS_LIMIT);
    CategoriesPageData {
        categories,
        top_tags,
        tag_preload_slugs,
        top_pornstars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::{CategoriesTemplate, RenderContext};
    use askama::Template;

    #[test]
    fn fixture_categories_page_renders_dynamic_cards_and_hooks() {
        let layout = SiteLayout::production();
        let data = categories_page_from_fixture_seed(layout.clone());
        assert!(!data.categories.is_empty());
        let html = CategoriesTemplate {
            ctx: RenderContext::categories_index(layout),
            categories: data.categories,
            top_tags: data.top_tags,
            tag_preload_slugs: data.tag_preload_slugs,
            top_pornstars: data.top_pornstars,
        }
        .render()
        .expect("categories render");
        assert!(html.contains(r#"id="search-genres-input""#));
        assert!(html.contains(r#"class="all_cats""#));
        assert!(html.contains(r#"class="thumb cat""#));
    }
}
