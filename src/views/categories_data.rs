//! View models and loaders for `/categories` (docs/pages/categories.md).

use crate::db::DbPool;
use crate::errors::AppError;
use crate::fixtures::{load_catalog_seed, seed_category_cards, seed_pornstar_cards};
use crate::models::entities::{list_top_pornstars_week, pornstar_profile_path, EntityIndexCard};
use crate::models::taxonomy::{
    list_categories_for_index, list_top_viewed_tags, CategoryCard, TagRow,
};
use crate::views::SiteLayout;

pub const CATEGORIES_TOP_TAGS_LIMIT: u32 = 29;
pub const CATEGORIES_TOP_PORNSTARS_LIMIT: u32 = 12;

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
