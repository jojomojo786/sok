//! Category and tag persistence models and query helpers.
//!
//! Table layout: **separate** `categories` and `tags` tables (see `docs/schema/taxonomy.md`).

use crate::db::DbPool;
use crate::errors::AppError;
use serde::Serialize;
use sqlx::FromRow;

pub const CATEGORY_THUMB_CDN_PREFIX: &str = "https://c.foxporn.tv/fox-images/categories/";

/// Resolved target for a single-segment `/{slug}` listing URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListingSlugKind {
    Category,
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingSort {
    Latest,
    MostViewed,
    MostCommented,
}

impl ListingSort {
    pub fn from_query(sort: Option<&str>) -> Self {
        match sort
            .map(str::trim)
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("mv") => Self::MostViewed,
            Some("mc") => Self::MostCommented,
            _ => Self::Latest,
        }
    }

    /// Map a slug-listing sort onto the `videos` query sort.
    pub fn to_video_list_sort(self) -> crate::models::video::VideoListSort {
        use crate::models::video::VideoListSort;
        match self {
            Self::Latest => VideoListSort::Newest,
            Self::MostViewed => VideoListSort::MostViewed,
            Self::MostCommented => VideoListSort::MostCommented,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchSort {
    #[default]
    Relevant,
    Newest,
    MostViewed,
    MostCommented,
}

impl SearchSort {
    pub fn from_query(sort: Option<&str>) -> Self {
        match sort
            .map(str::trim)
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("recent" | "new" | "newest") => Self::Newest,
            Some("mv") => Self::MostViewed,
            Some("mc") => Self::MostCommented,
            _ => Self::Relevant,
        }
    }

    pub fn to_video_list_sort(self) -> crate::models::video::VideoListSort {
        use crate::models::video::VideoListSort;
        match self {
            Self::Relevant => VideoListSort::Trending,
            Self::Newest => VideoListSort::Newest,
            Self::MostViewed => VideoListSort::MostViewed,
            Self::MostCommented => VideoListSort::MostCommented,
        }
    }
}

/// Query knobs for category/tag slug listing pages (`docs/pages/category-tag-listing.md`).
#[derive(Debug, Clone)]
pub struct SlugListingQuery {
    pub slug: String,
    pub sort: ListingSort,
    pub hd_only: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct CategoryRow {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub thumb_url: Option<String>,
    pub video_count: u32,
    pub intro_html: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, FromRow)]
pub struct TagRow {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub thumb_url: Option<String>,
    pub video_count: u32,
    pub weekly_views: u64,
    pub is_active: bool,
}

/// Category card for `/categories` grid (`.thumb.cat`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CategoryCard {
    pub slug: String,
    pub title: String,
    pub thumb_url: String,
    pub video_count: u32,
    pub listing_url: String,
}

/// JSON row for POST `/ajax/search_cats_tags_queries`.
#[derive(Debug, Clone, Serialize)]
pub struct CatsTagsSearchItem {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatsTagsSearchResponse {
    pub search_text: String,
    pub items: Vec<CatsTagsSearchItem>,
}

/// Header fields for a slug listing page.
#[derive(Debug, Clone)]
pub struct TaxonomyListingHeader {
    pub kind: ListingSlugKind,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub thumb_url: Option<String>,
    pub h1: String,
    pub canonical_path: String,
}

impl CategoryRow {
    pub fn default_thumb_url(slug: &str) -> String {
        format!("{CATEGORY_THUMB_CDN_PREFIX}{slug}.jpg")
    }

    pub fn effective_thumb_url(&self) -> String {
        self.thumb_url
            .clone()
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| Self::default_thumb_url(&self.slug))
    }

    pub fn to_card(&self) -> CategoryCard {
        let listing_url = format!("/{}", self.slug);
        CategoryCard {
            slug: self.slug.clone(),
            title: self.display_name.clone(),
            thumb_url: self.effective_thumb_url(),
            video_count: self.video_count,
            listing_url,
        }
    }

    pub fn to_listing_header(&self) -> TaxonomyListingHeader {
        let canonical_path = format!("/{}", self.slug);
        TaxonomyListingHeader {
            kind: ListingSlugKind::Category,
            slug: self.slug.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            thumb_url: Some(self.effective_thumb_url()),
            h1: format!("{} - Latest Porn Scenes", self.display_name),
            canonical_path,
        }
    }
}

impl TagRow {
    pub fn to_listing_header(&self) -> TaxonomyListingHeader {
        let canonical_path = format!("/{}", self.slug);
        TaxonomyListingHeader {
            kind: ListingSlugKind::Tag,
            slug: self.slug.clone(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            thumb_url: self.thumb_url.clone(),
            h1: format!("{} - Latest Porn Scenes", self.display_name),
            canonical_path,
        }
    }
}

/// Categories index: all active categories for `.all_cats` (`docs/pages/categories.md`).
pub async fn list_categories_for_index(pool: &DbPool) -> Result<Vec<CategoryCard>, AppError> {
    let rows: Vec<CategoryRow> = sqlx::query_as(
        r#"
        SELECT
            id, slug, display_name, description, thumb_url,
            video_count, intro_html, sort_order, is_active
        FROM categories
        WHERE is_active = 1
        ORDER BY sort_order ASC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.to_card()).collect())
}

pub async fn get_category_by_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<CategoryRow>, AppError> {
    let row: Option<CategoryRow> = sqlx::query_as(
        r#"
        SELECT
            id, slug, display_name, description, thumb_url,
            video_count,
            CAST(NULL AS CHAR) AS intro_html,
            sort_order,
            CAST(1 AS UNSIGNED) AS is_active
        FROM categories
        WHERE slug = ?
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn get_tag_by_slug(pool: &DbPool, slug: &str) -> Result<Option<TagRow>, AppError> {
    let row: Option<TagRow> = sqlx::query_as(
        r#"
        SELECT
            id, slug, display_name, description, thumb_url,
            video_count,
            CAST(0 AS UNSIGNED) AS weekly_views,
            CAST(1 AS UNSIGNED) AS is_active
        FROM tags
        WHERE slug = ?
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Resolve `/{slug}` to category or tag (category wins if both exist — should not happen with unique slugs per table).
pub async fn resolve_listing_slug(
    pool: &DbPool,
    slug: &str,
) -> Result<Option<(ListingSlugKind, TaxonomyListingHeader)>, AppError> {
    if let Some(cat) = get_category_by_slug(pool, slug).await? {
        return Ok(Some((ListingSlugKind::Category, cat.to_listing_header())));
    }
    if let Some(tag) = get_tag_by_slug(pool, slug).await? {
        return Ok(Some((ListingSlugKind::Tag, tag.to_listing_header())));
    }
    Ok(None)
}

/// Video IDs for a category listing (sort/HD applied when joining `videos` in the video model).
pub async fn list_video_ids_for_category(
    pool: &DbPool,
    category_id: u64,
    limit: u32,
    offset: u32,
) -> Result<Vec<u64>, AppError> {
    let ids: Vec<(u64,)> = sqlx::query_as(
        r#"
        SELECT vc.video_id
        FROM video_categories vc
        INNER JOIN categories c ON c.id = vc.category_id
        WHERE vc.category_id = ? AND c.is_active = 1
        ORDER BY vc.video_id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(category_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(ids.into_iter().map(|(id,)| id).collect())
}

pub async fn list_video_ids_for_tag(
    pool: &DbPool,
    tag_id: u64,
    limit: u32,
    offset: u32,
) -> Result<Vec<u64>, AppError> {
    let ids: Vec<(u64,)> = sqlx::query_as(
        r#"
        SELECT vt.video_id
        FROM video_tags vt
        INNER JOIN tags t ON t.id = vt.tag_id
        WHERE vt.tag_id = ? AND t.is_active = 1
        ORDER BY vt.video_id DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(tag_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(ids.into_iter().map(|(id,)| id).collect())
}

pub async fn list_tags_for_index(pool: &DbPool) -> Result<Vec<TagRow>, AppError> {
    let rows: Vec<TagRow> = sqlx::query_as(
        r#"
        SELECT
            id, slug, display_name, description, thumb_url,
            video_count, weekly_views, is_active
        FROM tags
        WHERE is_active = 1
        ORDER BY weekly_views DESC, display_name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Top viewed tags for categories page secondary section.
pub async fn list_top_viewed_tags(pool: &DbPool, limit: u32) -> Result<Vec<TagRow>, AppError> {
    let rows: Vec<TagRow> = sqlx::query_as(
        r#"
        SELECT
            id, slug, display_name, description, thumb_url,
            video_count, weekly_views, is_active
        FROM tags
        WHERE is_active = 1
        ORDER BY weekly_views DESC, display_name ASC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// In-page categories/tags search (`#search-genres-input` → `/ajax/search_cats_tags_queries`).
pub async fn search_categories_and_tags(
    pool: &DbPool,
    text: &str,
    limit: u32,
) -> Result<CatsTagsSearchResponse, AppError> {
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return Ok(CatsTagsSearchResponse {
            search_text: trimmed.to_string(),
            items: Vec::new(),
        });
    }

    let pattern = format!("%{}%", trimmed);
    let limit_i64 = i64::from(limit);

    let rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        (
            SELECT CAST(id AS CHAR) AS id, display_name AS name, CONCAT('/', slug) AS url
            FROM categories
            WHERE is_active = 1
              AND (
                display_name LIKE ?
                OR slug LIKE ?
                OR EXISTS (
                    SELECT 1 FROM taxonomy_search_aliases a
                    WHERE a.kind = 'category' AND a.entity_id = categories.id AND a.alias LIKE ?
                )
              )
        )
        UNION ALL
        (
            SELECT CAST(id AS CHAR) AS id, display_name AS name, CONCAT('/', slug) AS url
            FROM tags
            WHERE is_active = 1
              AND (
                display_name LIKE ?
                OR slug LIKE ?
                OR EXISTS (
                    SELECT 1 FROM taxonomy_search_aliases a
                    WHERE a.kind = 'tag' AND a.entity_id = tags.id AND a.alias LIKE ?
                )
              )
        )
        ORDER BY name ASC
        LIMIT ?
        "#,
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit_i64)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|(id, name, url)| CatsTagsSearchItem {
            id: id.to_string(),
            name,
            url,
        })
        .collect();

    Ok(CatsTagsSearchResponse {
        search_text: trimmed.to_string(),
        items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listing_sort_from_query_maps_mv_mc() {
        assert_eq!(ListingSort::from_query(Some("mv")), ListingSort::MostViewed);
        assert_eq!(
            ListingSort::from_query(Some("MC")),
            ListingSort::MostCommented
        );
        assert_eq!(ListingSort::from_query(None), ListingSort::Latest);
        assert_eq!(ListingSort::from_query(Some("")), ListingSort::Latest);
    }

    #[test]
    fn category_default_thumb_matches_cdn_pattern() {
        let row = CategoryRow {
            id: 1,
            slug: "milf".into(),
            display_name: "MILF".into(),
            description: None,
            thumb_url: None,
            video_count: 42,
            intro_html: None,
            sort_order: 0,
            is_active: true,
        };
        assert_eq!(
            row.effective_thumb_url(),
            "https://c.foxporn.tv/fox-images/categories/milf.jpg"
        );
        let card = row.to_card();
        assert_eq!(card.listing_url, "/milf");
        assert_eq!(card.video_count, 42);
        let header = row.to_listing_header();
        assert_eq!(header.h1, "MILF - Latest Porn Scenes");
        assert_eq!(header.canonical_path, "/milf");
    }

    #[test]
    fn tag_listing_header_matches_category_tag_listing_doc() {
        let tag = TagRow {
            id: 2,
            slug: "hot-mom".into(),
            display_name: "Hot Mom".into(),
            description: Some("desc".into()),
            thumb_url: None,
            video_count: 10,
            weekly_views: 100,
            is_active: true,
        };
        let header = tag.to_listing_header();
        assert_eq!(header.kind, ListingSlugKind::Tag);
        assert_eq!(header.h1, "Hot Mom - Latest Porn Scenes");
    }

    #[test]
    fn search_response_serializes_ajax_shape() {
        let resp = CatsTagsSearchResponse {
            search_text: "mil".into(),
            items: vec![CatsTagsSearchItem {
                id: "1".into(),
                name: "MILF".into(),
                url: "/milf".into(),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"search_text\":\"mil\""));
        assert!(json.contains("\"items\""));
        assert!(json.contains("/milf"));
    }
}
