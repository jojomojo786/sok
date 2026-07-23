//! Deterministic catalog fixtures for local development and tests.

use serde::Deserialize;
use sqlx::MySqlPool;

use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::entities::EntityIndexCard;
use crate::models::entities::ENTITY_SCHEMA_SQL;
use crate::models::taxonomy::{CategoryCard, CatsTagsSearchItem, CatsTagsSearchResponse};
use crate::models::video::VideoThumb;

pub const CATALOG_SEED_JSON: &str = include_str!("../../fixtures/catalog_seed.json");
pub const ENTITY_INDEX_LIVE_SEED_JSON: &str =
    include_str!("../../fixtures/entity_index_live_seed.json");
pub const DEV_CATALOG_SQL: &str = include_str!("../../sql/seeds/dev_catalog.sql");
pub const VIDEOS_SCHEMA_SQL: &str = include_str!("../../sql/schema/videos.sql");
pub const TAXONOMY_SCHEMA_SQL: &str = include_str!("../../migrations/001_taxonomy.sql");

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CatalogSeed {
    pub version: u32,
    pub source: Vec<String>,
    pub videos: Vec<SeedVideo>,
    pub categories: Vec<SeedCategory>,
    pub pornstars: Vec<SeedPornstar>,
    pub channels: Vec<SeedChannel>,
    pub links: SeedLinks,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedVideo {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub duration_seconds: u32,
    pub thumb_url: String,
    pub preview_mp4: String,
    pub views: u64,
    pub likes_up: u32,
    pub likes_down: u32,
    pub comment_count: u32,
    pub published_at: String,
    pub is_hd: u8,
    pub wide_thumb: u8,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedCategory {
    pub slug: String,
    pub display_name: String,
    pub thumb_url: String,
    pub video_count: u32,
    pub sort_order: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedPornstar {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub thumb_path: String,
    pub video_count: u32,
    pub week_views: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedChannel {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub thumb_path: String,
    pub video_count: u32,
    pub week_views: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EntityIndexLiveSeed {
    pub pornstars: Vec<SeedLiveEntityIndexCard>,
    pub channels: Vec<SeedLiveEntityIndexCard>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedLiveEntityIndexCard {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub thumb_path: String,
    pub video_count: u32,
    pub sort_order: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct SeedLinks {
    #[serde(default)]
    pub video_categories: Vec<SeedVideoCategoryLink>,
    #[serde(default)]
    pub video_pornstars: Vec<SeedVideoPornstarLink>,
    #[serde(default)]
    pub video_channels: Vec<SeedVideoChannelLink>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedVideoCategoryLink {
    pub video_id: u64,
    pub category_slug: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedVideoPornstarLink {
    pub video_id: u64,
    pub pornstar_slug: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct SeedVideoChannelLink {
    pub video_id: u64,
    pub channel_slug: String,
}

pub fn load_catalog_seed() -> Result<CatalogSeed, AppError> {
    serde_json::from_str(CATALOG_SEED_JSON)
        .map_err(|e| AppError::Internal(format!("invalid catalog seed JSON: {e}")))
}

pub fn load_entity_index_live_seed() -> Result<EntityIndexLiveSeed, AppError> {
    serde_json::from_str(ENTITY_INDEX_LIVE_SEED_JSON)
        .map_err(|e| AppError::Internal(format!("invalid entity index live seed JSON: {e}")))
}

fn live_entity_cards(cards: &[SeedLiveEntityIndexCard]) -> Vec<EntityIndexCard> {
    let mut cards: Vec<SeedLiveEntityIndexCard> = cards.to_vec();
    cards.sort_by_key(|card| card.sort_order);
    cards
        .into_iter()
        .map(|card| EntityIndexCard {
            id: card.id,
            slug: card.slug,
            display_name: card.display_name,
            thumb_path: card.thumb_path,
            video_count: card.video_count,
        })
        .collect()
}

pub fn live_pornstar_index_cards() -> Result<Vec<EntityIndexCard>, AppError> {
    let seed = load_entity_index_live_seed()?;
    Ok(live_entity_cards(&seed.pornstars))
}

pub fn live_channel_index_cards() -> Result<Vec<EntityIndexCard>, AppError> {
    let seed = load_entity_index_live_seed()?;
    Ok(live_entity_cards(&seed.channels))
}

pub fn seed_home_thumbs(seed: &CatalogSeed) -> Vec<VideoThumb> {
    seed.videos
        .iter()
        .map(|v| {
            let likes_percent = crate::models::video::like_percent(v.likes_up, v.likes_down);
            VideoThumb {
                id: v.id,
                slug: v.slug.clone(),
                title: v.title.clone(),
                duration_seconds: v.duration_seconds,
                thumb_url: v.thumb_url.clone(),
                preview_mp4: v.preview_mp4.clone(),
                views: v.views,
                likes_percent,
                comments: v.comment_count,
                published_at: chrono::NaiveDate::parse_from_str(&v.published_at, "%Y-%m-%d").ok(),
                is_hd: v.is_hd != 0,
                wide_thumb: v.wide_thumb != 0,
            }
        })
        .collect()
}

/// Video thumbs linked to a category slug (fixture fallback for `/{slug}`).

/// Video thumbs linked to a pornstar slug (fixture fallback for `/pornstar/{slug}`).
pub fn seed_thumbs_for_pornstar(seed: &CatalogSeed, pornstar_slug: &str) -> Vec<VideoThumb> {
    let ids: std::collections::HashSet<u64> = seed
        .links
        .video_pornstars
        .iter()
        .filter(|l| l.pornstar_slug == pornstar_slug)
        .map(|l| l.video_id)
        .collect();
    seed_home_thumbs(seed)
        .into_iter()
        .filter(|t| ids.contains(&t.id))
        .collect()
}

/// Video thumbs linked to a channel slug (fixture fallback for `/channel/{slug}`).
pub fn seed_thumbs_for_channel(seed: &CatalogSeed, channel_slug: &str) -> Vec<VideoThumb> {
    let ids: std::collections::HashSet<u64> = seed
        .links
        .video_channels
        .iter()
        .filter(|l| l.channel_slug == channel_slug)
        .map(|l| l.video_id)
        .collect();
    seed_home_thumbs(seed)
        .into_iter()
        .filter(|t| ids.contains(&t.id))
        .collect()
}

/// Video thumbs whose title matches a search needle (fixture fallback for `/videos/{term}`).
pub fn seed_thumbs_for_search(seed: &CatalogSeed, needle: &str) -> Vec<VideoThumb> {
    let n = needle.trim().to_ascii_lowercase();
    if n.len() < crate::models::search::SEARCH_MIN_QUERY_LEN {
        return Vec::new();
    }
    seed_home_thumbs(seed)
        .into_iter()
        .filter(|t| {
            t.title.to_ascii_lowercase().contains(&n) || t.slug.contains(&n.replace(' ', "-"))
        })
        .collect()
}

pub fn seed_thumbs_for_category(seed: &CatalogSeed, category_slug: &str) -> Vec<VideoThumb> {
    let ids: std::collections::HashSet<u64> = seed
        .links
        .video_categories
        .iter()
        .filter(|l| l.category_slug == category_slug)
        .map(|l| l.video_id)
        .collect();
    seed_home_thumbs(seed)
        .into_iter()
        .filter(|t| ids.contains(&t.id))
        .collect()
}

pub fn seed_category_cards(seed: &CatalogSeed) -> Vec<CategoryCard> {
    seed.categories
        .iter()
        .map(|c| CategoryCard {
            slug: c.slug.clone(),
            title: c.display_name.clone(),
            thumb_url: c.thumb_url.clone(),
            video_count: c.video_count,
            listing_url: format!("/{}", c.slug),
            link_title: None,
            alt_text: format!("{} porn videos", c.display_name),
            lazy: false,
            uses_tags_icon: false,
        })
        .collect()
}

/// In-memory search for `/ajax/search_cats_tags_queries` when DB is empty or unavailable.
pub fn search_categories_and_tags_from_seed(
    seed: &CatalogSeed,
    text: &str,
    limit: u32,
) -> CatsTagsSearchResponse {
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return CatsTagsSearchResponse {
            search_text: trimmed.to_string(),
            items: Vec::new(),
        };
    }

    let needle = trimmed.to_lowercase();
    let mut items: Vec<CatsTagsSearchItem> = seed
        .categories
        .iter()
        .filter(|c| {
            c.display_name.to_lowercase().contains(&needle)
                || c.slug.to_lowercase().contains(&needle)
        })
        .map(|c| CatsTagsSearchItem {
            id: format!("seed-{}", c.sort_order),
            name: c.display_name.clone(),
            url: format!("/{}", c.slug),
        })
        .collect();

    items.sort_by(|a, b| a.name.cmp(&b.name));
    items.truncate(limit as usize);

    CatsTagsSearchResponse {
        search_text: trimmed.to_string(),
        items,
    }
}

pub fn seed_pornstar_cards(seed: &CatalogSeed) -> Vec<EntityIndexCard> {
    seed.pornstars
        .iter()
        .map(|p| EntityIndexCard {
            id: p.id,
            slug: p.slug.clone(),
            display_name: p.display_name.clone(),
            thumb_path: p.thumb_path.clone(),
            video_count: p.video_count,
        })
        .collect()
}

pub fn seed_top_pornstars_week(seed: &CatalogSeed, limit: u32) -> Vec<EntityIndexCard> {
    let mut cards = seed_pornstar_cards(seed);
    cards.sort_by(|a, b| {
        seed.pornstars
            .iter()
            .find(|p| p.slug == a.slug)
            .map(|p| p.week_views)
            .unwrap_or(0)
            .cmp(
                &seed
                    .pornstars
                    .iter()
                    .find(|p| p.slug == b.slug)
                    .map(|p| p.week_views)
                    .unwrap_or(0),
            )
            .then_with(|| b.video_count.cmp(&a.video_count))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    cards.truncate(limit as usize);
    cards
}

pub fn seed_top_channels_week(seed: &CatalogSeed, limit: u32) -> Vec<EntityIndexCard> {
    let mut cards = seed_channel_cards(seed);
    cards.sort_by(|a, b| {
        seed.channels
            .iter()
            .find(|c| c.slug == a.slug)
            .map(|c| c.week_views)
            .unwrap_or(0)
            .cmp(
                &seed
                    .channels
                    .iter()
                    .find(|c| c.slug == b.slug)
                    .map(|c| c.week_views)
                    .unwrap_or(0),
            )
            .then_with(|| b.video_count.cmp(&a.video_count))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    cards.truncate(limit as usize);
    cards
}

pub fn seed_top_viewed_tags(
    seed: &CatalogSeed,
    limit: u32,
) -> Vec<crate::models::taxonomy::TagRow> {
    let mut cards = seed_category_cards(seed);
    cards.sort_by(|a, b| {
        b.video_count
            .cmp(&a.video_count)
            .then_with(|| a.title.cmp(&b.title))
    });
    cards.truncate(limit as usize);
    cards
        .into_iter()
        .enumerate()
        .map(|(idx, card)| crate::models::taxonomy::TagRow {
            id: (idx + 1) as u64,
            slug: card.slug,
            display_name: card.title,
            description: None,
            thumb_url: Some(card.thumb_url),
            video_count: card.video_count,
            weekly_views: u64::from(card.video_count),
            is_active: true,
        })
        .collect()
}

pub fn seed_channel_cards(seed: &CatalogSeed) -> Vec<EntityIndexCard> {
    seed.channels
        .iter()
        .map(|c| EntityIndexCard {
            id: c.id,
            slug: c.slug.clone(),
            display_name: c.title.clone(),
            thumb_path: c.thumb_path.clone(),
            video_count: c.video_count,
        })
        .collect()
}

async fn maybe_reset_dev_catalog_tables(pool: &DbPool) -> Result<(), AppError> {
    if std::env::var("SOK_FIXTURES_RESET").ok().as_deref() != Some("1") {
        return Ok(());
    }
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(pool)
        .await?;
    for table in [
        "video_channels",
        "video_pornstars",
        "video_categories",
        "video_tags",
        "videos",
        "categories",
        "tags",
        "taxonomy_search_aliases",
        "pornstar_aliases",
        "channel_aliases",
        "pornstars",
        "channels",
    ] {
        let sql = format!("DROP TABLE IF EXISTS {table}");
        sqlx::query(&sql).execute(pool).await?;
    }
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn ensure_dev_catalog_schema(pool: &DbPool) -> Result<(), AppError> {
    maybe_reset_dev_catalog_tables(pool).await?;
    for sql in [VIDEOS_SCHEMA_SQL, TAXONOMY_SCHEMA_SQL, ENTITY_SCHEMA_SQL] {
        execute_sql_script(pool, sql).await?;
    }
    Ok(())
}

pub async fn apply_catalog_seed(pool: &DbPool, seed: &CatalogSeed) -> Result<(), AppError> {
    ensure_dev_catalog_schema(pool).await?;

    let mut tx = pool.begin().await?;
    clear_catalog_tables(&mut tx).await?;

    for c in &seed.categories {
        sqlx::query(
            "INSERT INTO categories (slug, display_name, thumb_url, video_count, sort_order) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&c.slug)
        .bind(&c.display_name)
        .bind(&c.thumb_url)
        .bind(c.video_count)
        .bind(c.sort_order)
        .execute(&mut *tx)
        .await?;
    }

    for p in &seed.pornstars {
        sqlx::query(
            "INSERT INTO pornstars (id, slug, display_name, thumb_path, video_count, week_views) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(p.id)
        .bind(&p.slug)
        .bind(&p.display_name)
        .bind(&p.thumb_path)
        .bind(p.video_count)
        .bind(p.week_views)
        .execute(&mut *tx)
        .await?;
    }

    for ch in &seed.channels {
        sqlx::query(
            "INSERT INTO channels (id, slug, title, thumb_path, video_count, week_views) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(ch.id)
        .bind(&ch.slug)
        .bind(&ch.title)
        .bind(&ch.thumb_path)
        .bind(ch.video_count)
        .bind(ch.week_views)
        .execute(&mut *tx)
        .await?;
    }

    for v in &seed.videos {
        sqlx::query(
            "INSERT INTO videos (id, slug, title, duration_seconds, thumb_url, preview_mp4, views, likes_up, likes_down, comment_count, published_at, is_hd, wide_thumb, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'published')",
        )
        .bind(v.id)
        .bind(&v.slug)
        .bind(&v.title)
        .bind(v.duration_seconds)
        .bind(&v.thumb_url)
        .bind(&v.preview_mp4)
        .bind(v.views)
        .bind(v.likes_up)
        .bind(v.likes_down)
        .bind(v.comment_count)
        .bind(&v.published_at)
        .bind(v.is_hd)
        .bind(v.wide_thumb)
        .execute(&mut *tx)
        .await?;
    }

    let category_ids: Vec<(String, i64)> = sqlx::query_as("SELECT slug, id FROM categories")
        .fetch_all(&mut *tx)
        .await?;
    let cat_map: std::collections::HashMap<String, i64> =
        category_ids.into_iter().map(|(s, id)| (s, id)).collect();

    for link in &seed.links.video_categories {
        let category_id = cat_map.get(&link.category_slug).ok_or_else(|| {
            AppError::Internal(format!(
                "seed link references unknown category slug: {}",
                link.category_slug
            ))
        })?;
        sqlx::query("INSERT INTO video_categories (video_id, category_id) VALUES (?, ?)")
            .bind(link.video_id)
            .bind(category_id)
            .execute(&mut *tx)
            .await?;
    }

    let pornstar_ids: Vec<(String, i64)> = sqlx::query_as("SELECT slug, id FROM pornstars")
        .fetch_all(&mut *tx)
        .await?;
    let ps_map: std::collections::HashMap<String, i64> =
        pornstar_ids.into_iter().map(|(s, id)| (s, id)).collect();

    for link in &seed.links.video_pornstars {
        let pornstar_id = ps_map.get(&link.pornstar_slug).ok_or_else(|| {
            AppError::Internal(format!(
                "seed link references unknown pornstar slug: {}",
                link.pornstar_slug
            ))
        })?;
        sqlx::query("INSERT INTO video_pornstars (video_id, pornstar_id) VALUES (?, ?)")
            .bind(link.video_id)
            .bind(pornstar_id)
            .execute(&mut *tx)
            .await?;
    }

    let channel_ids: Vec<(String, i64)> = sqlx::query_as("SELECT slug, id FROM channels")
        .fetch_all(&mut *tx)
        .await?;
    let ch_map: std::collections::HashMap<String, i64> =
        channel_ids.into_iter().map(|(s, id)| (s, id)).collect();

    for link in &seed.links.video_channels {
        let channel_id = ch_map.get(&link.channel_slug).ok_or_else(|| {
            AppError::Internal(format!(
                "seed link references unknown channel slug: {}",
                link.channel_slug
            ))
        })?;
        sqlx::query("INSERT INTO video_channels (video_id, channel_id) VALUES (?, ?)")
            .bind(link.video_id)
            .bind(channel_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn apply_default_catalog_seed(pool: &DbPool) -> Result<CatalogSeed, AppError> {
    let seed = load_catalog_seed()?;
    apply_catalog_seed(pool, &seed).await?;
    Ok(seed)
}

async fn clear_catalog_tables(tx: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<(), AppError> {
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut **tx)
        .await?;
    for table in [
        "video_channels",
        "video_pornstars",
        "video_categories",
        "videos",
        "categories",
        "pornstars",
        "channels",
    ] {
        let sql = format!("TRUNCATE TABLE {table}");
        sqlx::query(&sql).execute(&mut **tx).await?;
    }
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn execute_sql_script(pool: &MySqlPool, script: &str) -> Result<(), AppError> {
    for stmt in script.split(';') {
        let stmt = stmt.trim();
        if stmt.is_empty() || stmt.starts_with("--") {
            continue;
        }
        sqlx::query(stmt).execute(pool).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::video::{list_home_thumbs, VideoListSort, DEFAULT_THUMBS_CDN};

    #[test]
    fn catalog_seed_json_is_non_empty_and_deterministic() {
        let seed = load_catalog_seed().expect("seed json");
        assert_eq!(seed.version, 1);
        assert!(!seed.videos.is_empty());
        assert!(!seed.categories.is_empty());
        assert!(!seed.pornstars.is_empty());
        assert!(!seed.channels.is_empty());
        assert!(seed_home_thumbs(&seed)[0]
            .thumb_url
            .contains(DEFAULT_THUMBS_CDN));
    }

    #[test]
    fn fixture_fallback_search_matches_production_json_keys() {
        let seed = load_catalog_seed().expect("seed json");
        let resp = search_categories_and_tags_from_seed(&seed, "big", 50);
        assert_eq!(resp.search_text, "big");
        assert!(!resp.items.is_empty());
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["items"][0].get("id").is_some());
        assert!(json["items"][0].get("name").is_some());
        assert!(json["items"][0].get("url").is_some());
    }

    #[test]
    fn dev_catalog_sql_references_representative_slugs() {
        assert!(DEV_CATALOG_SQL.contains("INSERT INTO categories"));
        assert!(DEV_CATALOG_SQL.contains("INSERT INTO pornstars"));
    }

    #[tokio::test]
    async fn apply_seed_populates_query_surfaces_when_database_available() {
        dotenv::dotenv().ok();
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) if !url.trim().is_empty() => url,
            _ => return,
        };
        if std::env::var("SOK_SKIP_DB_SEED_TESTS").ok().as_deref() == Some("1") {
            return;
        }

        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(2)
            .connect(&database_url)
            .await
            .expect("db connect");

        std::env::set_var("SOK_FIXTURES_RESET", "1");
        let seed = match apply_default_catalog_seed(&pool).await {
            Ok(seed) => seed,
            Err(crate::errors::AppError::Db(e)) => {
                eprintln!("skipping DB seed integration (schema/db): {e}");
                pool.close().await;
                return;
            }
            Err(e) => panic!("apply seed: {e}"),
        };

        let home = list_home_thumbs(&pool, 1, 12, VideoListSort::Trending, false)
            .await
            .expect("home thumbs");
        assert!(!home.is_empty());
        assert_eq!(home[0].slug, seed.videos[0].slug);

        let categories = crate::models::taxonomy::list_categories_for_index(&pool)
            .await
            .expect("categories");
        assert!(!categories.is_empty());

        let pornstars =
            crate::models::entities::list_pornstars_index(&pool, Default::default(), 1, 12)
                .await
                .expect("pornstars");
        assert!(!pornstars.items.is_empty());

        pool.close().await;
    }
}
