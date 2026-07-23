//! Pornstar and channel catalog models, media URL helpers, and sqlx queries.
//!
//! Schema alignment: `docs/pages/pornstars.md`, `pornstar-profile.md`, `channels.md`,
//! `channel-profile.md`, and header `search_help` (`docs/pages/search.md`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::errors::AppError;
use crate::models::taxonomy::ListingSort;
use crate::models::video::VideoListSort;

/// CDN host used by mirrored templates (`c.foxporn.tv`).
pub use crate::config::DEFAULT_MEDIA_CDN;

/// Relative image roots on the CDN.
pub const PORNSTAR_THUMB_DIR: &str = "fox-images/pornstars";
pub const CHANNEL_THUMB_DIR: &str = "fox-images/channels";

/// Default page size for `/pornstars` and `/channels` grids.
pub const ENTITY_INDEX_PAGE_SIZE: u32 = crate::models::pagination::DEFAULT_ENTITY_INDEX_PER_PAGE;

/// Sort options shared by pornstar and channel index/profile listings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityListSort {
    #[default]
    Trending,
    NameAsc,
    NameDesc,
    VideoCountDesc,
    VideoCountAsc,
    Newest,
}

impl EntityListSort {
    pub fn from_query_param(value: Option<&str>) -> Self {
        match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("name" | "name_asc" | "alpha") => Self::NameAsc,
            Some("name_desc") => Self::NameDesc,
            Some("videos" | "video_count" | "count") => Self::VideoCountDesc,
            Some("videos_asc" | "video_count_asc") => Self::VideoCountAsc,
            Some("new" | "newest") => Self::Newest,
            _ => Self::Trending,
        }
    }

    fn order_sql_pornstars(self) -> &'static str {
        match self {
            Self::Trending => "p.week_views DESC, p.video_count DESC, p.display_name ASC",
            Self::NameAsc => "p.display_name ASC",
            Self::NameDesc => "p.display_name DESC",
            Self::VideoCountDesc => "p.video_count DESC, p.display_name ASC",
            Self::VideoCountAsc => "p.video_count ASC, p.display_name ASC",
            Self::Newest => "p.created_at DESC, p.display_name ASC",
        }
    }

    fn order_sql_channels(self) -> &'static str {
        match self {
            Self::Trending => "c.week_views DESC, c.video_count DESC, c.title ASC",
            Self::NameAsc => "c.title ASC",
            Self::NameDesc => "c.title DESC",
            Self::VideoCountDesc => "c.video_count DESC, c.title ASC",
            Self::VideoCountAsc => "c.video_count ASC, c.title ASC",
            Self::Newest => "c.created_at DESC, c.title ASC",
        }
    }
}

/// Shared list-card fields for `.thumb.cat` grids and AJAX refresh widgets.
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct EntityIndexCard {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub thumb_path: String,
    pub video_count: u32,
}

/// Full pornstar profile row (`pornstars` table).
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Pornstar {
    pub id: u64,
    pub slug: String,
    pub display_name: String,
    pub thumb_path: String,
    pub banner_path: Option<String>,
    pub avatar_path: Option<String>,
    pub bio: Option<String>,
    pub video_count: u32,
    pub verified: bool,
    pub week_views: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Full channel profile row (`channels` table).
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Channel {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub thumb_path: String,
    pub logo_path: Option<String>,
    pub banner_path: Option<String>,
    pub bio: Option<String>,
    pub video_count: u32,
    pub network_name: Option<String>,
    pub week_views: u64,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Alternate searchable names (`pornstar_aliases` / `channel_aliases`).
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct EntityAlias {
    pub id: u64,
    pub alias: String,
}

/// In-page pornstars search item (`/ajax/search_{type}`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PornstarPageSearchItem {
    pub url: String,
    pub thumb: String,
    pub orig_name: String,
    pub count_videos: u32,
}

/// Header search_help pornstar hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHelpPornstar {
    pub url_pornstar: String,
    pub orig_name: String,
    pub thumb: String,
}

/// Header search_help channel hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHelpChannel {
    pub url: String,
    pub orig_name: String,
    pub thumb: String,
}

/// Lightweight video row for entity profile grids (joins `videos` + link tables).
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct EntityVideoThumb {
    pub video_id: u64,
    pub slug: String,
    pub title: String,
    pub thumb_path: String,
    pub preview_path: Option<String>,
    pub duration_seconds: u32,
    pub views: u64,
    pub likes_pct: u8,
    pub comments_count: u32,
    pub published_at: Option<DateTime<Utc>>,
    pub wide_thumb: bool,
}

#[derive(Debug, Clone)]
pub struct EntityIndexPage<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

// --- Media URL helpers (shared between pornstars and channels) ---

pub fn media_url(cdn_base: &str, path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return String::new();
    }
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    format!(
        "{}/{}",
        cdn_base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn pornstar_thumb_url(cdn_base: &str, slug: &str) -> String {
    media_url(cdn_base, &format!("{PORNSTAR_THUMB_DIR}/{slug}.jpg"))
}

pub fn channel_thumb_url(cdn_base: &str, slug: &str) -> String {
    media_url(cdn_base, &format!("{CHANNEL_THUMB_DIR}/{slug}.jpg"))
}

pub fn pornstar_profile_path(slug: &str) -> String {
    format!("/pornstar/{slug}")
}

pub fn channel_profile_path(slug: &str) -> String {
    format!("/channel/{slug}")
}

pub fn pornstar_banner_url(cdn_base: &str, slug: &str) -> String {
    media_url(
        cdn_base,
        &format!("{PORNSTAR_THUMB_DIR}/b-bn-mdl-{slug}.jpg"),
    )
}

pub fn channel_banner_url(cdn_base: &str, slug: &str) -> String {
    media_url(
        cdn_base,
        &format!("{CHANNEL_THUMB_DIR}/b-bn-chnl-{slug}.jpg"),
    )
}

fn entity_video_order_sql(sort: VideoListSort) -> &'static str {
    match sort {
        VideoListSort::MostViewed => "v.views DESC, v.id DESC",
        VideoListSort::MostCommented => "v.comments_count DESC, v.views DESC, v.id DESC",
        VideoListSort::Newest | VideoListSort::Trending => {
            "v.published_at DESC, v.views DESC, v.id DESC"
        }
    }
}

pub fn listing_sort_to_entity_video_sort(sort: ListingSort) -> VideoListSort {
    sort.to_video_list_sort()
}

fn sort_label_for_profile(sort: ListingSort) -> &'static str {
    match sort {
        ListingSort::MostViewed => "Most Viewed Videos",
        ListingSort::MostCommented => "Most Commented Videos",
        ListingSort::Latest => "Newest Videos",
    }
}

fn channel_sort_label_for_profile(sort: ListingSort) -> &'static str {
    match sort {
        ListingSort::MostViewed => "Most Viewed Videos",
        ListingSort::MostCommented => "Most Commented Videos",
        ListingSort::Latest => "Latest Videos",
    }
}

/// Profile listing header shared by pornstar and channel pages (H1 + SEO fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityProfileHeader {
    pub slug: String,
    pub display_name: String,
    pub h1: String,
    pub title: String,
    pub description: String,
    pub banner_url: Option<String>,
    pub avatar_url: Option<String>,
    pub show_verified_badge: bool,
    pub avatar_alt_suffix: &'static str,
}

impl EntityProfileHeader {
    pub fn for_pornstar(p: &Pornstar, cdn_base: &str, sort: ListingSort) -> Self {
        let name = p.display_name.clone();
        Self {
            slug: p.slug.clone(),
            display_name: name.clone(),
            h1: format!("{name} - {}", sort_label_for_profile(sort)),
            title: format!("{name} Free Porn Videos and Scenes | PornsOK.com"),
            description: format!(
                "Watch free {name} porn videos on PornsOK.com. Stream the newest {name} XXX scenes in HD every day."
            ),
            banner_url: p
                .banner_url(cdn_base)
                .or_else(|| Some(pornstar_banner_url(cdn_base, &p.slug))),
            avatar_url: p.avatar_url(cdn_base),
            show_verified_badge: p.verified,
            avatar_alt_suffix: "pornstar",
        }
    }

    pub fn for_channel(c: &Channel, cdn_base: &str, sort: ListingSort) -> Self {
        let name = c.title.clone();
        Self {
            slug: c.slug.clone(),
            display_name: name.clone(),
            h1: format!("{name} - {}", channel_sort_label_for_profile(sort)),
            title: format!("{name} Porn Channel - Free Sex Videos | PornsOK.com"),
            description: format!(
                "Watch free {name} porn channel videos on PornsOK.com. Browse the latest {name} sex movies and scenes in HD."
            ),
            banner_url: c
                .banner_url(cdn_base)
                .or_else(|| Some(channel_banner_url(cdn_base, &c.slug))),
            avatar_url: Some(c.logo_url(cdn_base)),
            show_verified_badge: true,
            avatar_alt_suffix: "pornstar",
        }
    }
}

impl EntityIndexCard {
    pub fn thumb_url(&self, cdn_base: &str) -> String {
        if self.thumb_path.is_empty() {
            return String::new();
        }
        media_url(cdn_base, &self.thumb_path)
    }
}

impl Pornstar {
    pub fn thumb_url(&self, cdn_base: &str) -> String {
        if self.thumb_path.is_empty() {
            pornstar_thumb_url(cdn_base, &self.slug)
        } else {
            media_url(cdn_base, &self.thumb_path)
        }
    }

    pub fn banner_url(&self, cdn_base: &str) -> Option<String> {
        self.banner_path.as_ref().map(|p| media_url(cdn_base, p))
    }

    pub fn avatar_url(&self, cdn_base: &str) -> Option<String> {
        self.avatar_path
            .as_ref()
            .map(|p| media_url(cdn_base, p))
            .or_else(|| Some(self.thumb_url(cdn_base)))
    }

    pub fn profile_url(&self) -> String {
        pornstar_profile_path(&self.slug)
    }
}

impl Channel {
    pub fn thumb_url(&self, cdn_base: &str) -> String {
        if self.thumb_path.is_empty() {
            channel_thumb_url(cdn_base, &self.slug)
        } else {
            media_url(cdn_base, &self.thumb_path)
        }
    }

    pub fn logo_url(&self, cdn_base: &str) -> String {
        self.logo_path
            .as_ref()
            .map(|p| media_url(cdn_base, p))
            .unwrap_or_else(|| self.thumb_url(cdn_base))
    }

    pub fn banner_url(&self, cdn_base: &str) -> Option<String> {
        self.banner_path.as_ref().map(|p| media_url(cdn_base, p))
    }

    pub fn profile_url(&self) -> String {
        channel_profile_path(&self.slug)
    }
}

impl EntityVideoThumb {
    pub fn thumb_url(&self, cdn_base: &str) -> String {
        media_url(cdn_base, &self.thumb_path)
    }

    pub fn preview_mp4_url(&self, cdn_base: &str) -> Option<String> {
        self.preview_path.as_ref().map(|p| media_url(cdn_base, p))
    }

    pub fn video_page_path(&self) -> String {
        format!("/video/{}.html", self.slug)
    }

    pub fn to_video_thumb(&self, cdn_base: &str) -> crate::models::video::VideoThumb {
        crate::models::video::VideoThumb {
            id: self.video_id,
            slug: self.slug.clone(),
            title: self.title.clone(),
            duration_seconds: self.duration_seconds,
            thumb_url: self.thumb_url(cdn_base),
            preview_mp4: self.preview_mp4_url(cdn_base).unwrap_or_default(),
            views: self.views,
            likes_percent: self.likes_pct,
            comments: self.comments_count,
            published_at: self.published_at.map(|d| d.date_naive()),
            is_hd: false,
            wide_thumb: self.wide_thumb,
        }
    }
}

impl From<EntityIndexCard> for PornstarPageSearchItem {
    fn from(card: EntityIndexCard) -> Self {
        Self {
            url: pornstar_profile_path(&card.slug),
            thumb: card.thumb_path,
            orig_name: card.display_name,
            count_videos: card.video_count,
        }
    }
}

// --- Pornstar queries ---

pub async fn count_pornstars(pool: &MySqlPool) -> Result<u64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pornstars")
        .fetch_one(pool)
        .await?;
    Ok(row.0.max(0) as u64)
}

pub async fn list_pornstars_index(
    pool: &MySqlPool,
    sort: EntityListSort,
    page: u32,
    per_page: u32,
) -> Result<EntityIndexPage<EntityIndexCard>, AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 200);
    let offset = (page - 1) as i64 * per_page as i64;
    let total = count_pornstars(pool).await?;

    let order = sort.order_sql_pornstars();
    let sql = format!(
        "SELECT p.id, p.slug, p.display_name, p.thumb_path, p.video_count
         FROM pornstars p
         ORDER BY {order}
         LIMIT ? OFFSET ?"
    );

    let items = sqlx::query_as::<_, EntityIndexCard>(&sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(EntityIndexPage {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn pornstar_by_slug(pool: &MySqlPool, slug: &str) -> Result<Option<Pornstar>, AppError> {
    let row = sqlx::query_as::<_, Pornstar>(
        "SELECT id, slug, display_name, thumb_path, banner_path, avatar_path, bio,
                video_count, verified, week_views, created_at, updated_at
         FROM pornstars
         WHERE slug = ?
         LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn pornstar_aliases(
    pool: &MySqlPool,
    pornstar_id: u64,
) -> Result<Vec<EntityAlias>, AppError> {
    let rows = sqlx::query_as::<_, EntityAlias>(
        "SELECT id, alias FROM pornstar_aliases WHERE pornstar_id = ? ORDER BY alias ASC",
    )
    .bind(pornstar_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_top_pornstars_week(
    pool: &MySqlPool,
    limit: u32,
) -> Result<Vec<EntityIndexCard>, AppError> {
    let limit = limit.clamp(1, 100);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT id, slug, display_name, thumb_path, video_count
         FROM pornstars
         ORDER BY week_views DESC, video_count DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn search_pornstars_for_page(
    pool: &MySqlPool,
    query: &str,
    limit: u32,
) -> Result<Vec<PornstarPageSearchItem>, AppError> {
    let needle = format!("%{}%", normalize_search_needle(query));
    let limit = limit.clamp(1, 200);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT DISTINCT p.id, p.slug, p.display_name, p.thumb_path, p.video_count
         FROM pornstars p
         LEFT JOIN pornstar_aliases a ON a.pornstar_id = p.id
         WHERE p.display_name LIKE ? OR a.alias LIKE ? OR p.slug LIKE ?
         ORDER BY p.video_count DESC, p.display_name ASC
         LIMIT ?",
    )
    .bind(&needle)
    .bind(&needle)
    .bind(&needle)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(PornstarPageSearchItem::from).collect())
}

pub async fn search_pornstars_for_help(
    pool: &MySqlPool,
    query: &str,
    limit: u32,
    cdn_base: &str,
) -> Result<Vec<SearchHelpPornstar>, AppError> {
    let needle = format!("%{}%", normalize_search_needle(query));
    let limit = limit.clamp(1, 50);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT DISTINCT p.id, p.slug, p.display_name, p.thumb_path, p.video_count
         FROM pornstars p
         LEFT JOIN pornstar_aliases a ON a.pornstar_id = p.id
         WHERE p.display_name LIKE ? OR a.alias LIKE ? OR p.slug LIKE ?
         ORDER BY p.video_count DESC, p.display_name ASC
         LIMIT ?",
    )
    .bind(&needle)
    .bind(&needle)
    .bind(&needle)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let thumb = row.thumb_url(cdn_base);
            SearchHelpPornstar {
                url_pornstar: pornstar_profile_path(&row.slug),
                orig_name: row.display_name,
                thumb,
            }
        })
        .collect())
}

pub async fn count_videos_for_pornstar(
    pool: &MySqlPool,
    slug: &str,
    hd_only: bool,
) -> Result<u64, AppError> {
    let sql = if hd_only {
        "SELECT COUNT(DISTINCT v.id)
         FROM videos v
         INNER JOIN video_pornstars vp ON vp.video_id = v.id
         INNER JOIN pornstars p ON p.id = vp.pornstar_id
         WHERE p.slug = ? AND v.is_active = 1 AND v.is_hd = 1"
    } else {
        "SELECT COUNT(DISTINCT v.id)
         FROM videos v
         INNER JOIN video_pornstars vp ON vp.video_id = v.id
         INNER JOIN pornstars p ON p.id = vp.pornstar_id
         WHERE p.slug = ? AND v.is_active = 1"
    };
    let row: (i64,) = sqlx::query_as(sql).bind(slug).fetch_one(pool).await?;
    Ok(row.0.max(0) as u64)
}

pub async fn videos_for_pornstar(
    pool: &MySqlPool,
    slug: &str,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<EntityVideoThumb>, AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 120);
    let offset = (page - 1) as i64 * per_page as i64;
    let order = entity_video_order_sql(sort);
    let sql = if hd_only {
        format!(
            "SELECT v.id AS video_id, v.slug, v.title, v.thumb_path, v.preview_path,
                v.duration_seconds, v.views, v.likes_pct, v.comments_count,
                v.published_at, v.wide_thumb
         FROM videos v
         INNER JOIN video_pornstars vp ON vp.video_id = v.id
         INNER JOIN pornstars p ON p.id = vp.pornstar_id
         WHERE p.slug = ? AND v.is_active = 1 AND v.is_hd = 1
         ORDER BY {order}
         LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT v.id AS video_id, v.slug, v.title, v.thumb_path, v.preview_path,
                v.duration_seconds, v.views, v.likes_pct, v.comments_count,
                v.published_at, v.wide_thumb
         FROM videos v
         INNER JOIN video_pornstars vp ON vp.video_id = v.id
         INNER JOIN pornstars p ON p.id = vp.pornstar_id
         WHERE p.slug = ? AND v.is_active = 1
         ORDER BY {order}
         LIMIT ? OFFSET ?"
        )
    };

    let rows = sqlx::query_as::<_, EntityVideoThumb>(&sql)
        .bind(slug)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

// --- Channel queries ---

pub async fn count_channels(pool: &MySqlPool) -> Result<u64, AppError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channels")
        .fetch_one(pool)
        .await?;
    Ok(row.0.max(0) as u64)
}

pub async fn list_channels_index(
    pool: &MySqlPool,
    sort: EntityListSort,
    page: u32,
    per_page: u32,
) -> Result<EntityIndexPage<EntityIndexCard>, AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 200);
    let offset = (page - 1) as i64 * per_page as i64;
    let total = count_channels(pool).await?;

    let order = sort.order_sql_channels();
    let sql = format!(
        "SELECT c.id, c.slug, c.title AS display_name, c.thumb_path, c.video_count
         FROM channels c
         ORDER BY {order}
         LIMIT ? OFFSET ?"
    );

    let items = sqlx::query_as::<_, EntityIndexCard>(&sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(EntityIndexPage {
        items,
        total,
        page,
        per_page,
    })
}

pub async fn channel_by_slug(pool: &MySqlPool, slug: &str) -> Result<Option<Channel>, AppError> {
    let row = sqlx::query_as::<_, Channel>(
        "SELECT id, slug, title, thumb_path, logo_path, banner_path, bio,
                video_count, network_name, week_views, created_at, updated_at
         FROM channels
         WHERE slug = ?
         LIMIT 1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn channel_aliases(
    pool: &MySqlPool,
    channel_id: u64,
) -> Result<Vec<EntityAlias>, AppError> {
    let rows = sqlx::query_as::<_, EntityAlias>(
        "SELECT id, alias FROM channel_aliases WHERE channel_id = ? ORDER BY alias ASC",
    )
    .bind(channel_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn list_top_channels_week(
    pool: &MySqlPool,
    limit: u32,
) -> Result<Vec<EntityIndexCard>, AppError> {
    let limit = limit.clamp(1, 100);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT id, slug, title AS display_name, thumb_path, video_count
         FROM channels
         ORDER BY week_views DESC, video_count DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn search_channels_for_page(
    pool: &MySqlPool,
    query: &str,
    limit: u32,
) -> Result<Vec<PornstarPageSearchItem>, AppError> {
    let needle = format!("%{}%", normalize_search_needle(query));
    let limit = limit.clamp(1, 200);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT DISTINCT c.id, c.slug, c.title AS display_name, c.thumb_path, c.video_count
         FROM channels c
         LEFT JOIN channel_aliases a ON a.channel_id = c.id
         WHERE c.title LIKE ? OR a.alias LIKE ? OR c.slug LIKE ?
         ORDER BY c.video_count DESC, c.title ASC
         LIMIT ?",
    )
    .bind(&needle)
    .bind(&needle)
    .bind(&needle)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PornstarPageSearchItem {
            url: channel_profile_path(&row.slug),
            thumb: row.thumb_path,
            orig_name: row.display_name,
            count_videos: row.video_count,
        })
        .collect())
}

pub async fn search_channels_for_help(
    pool: &MySqlPool,
    query: &str,
    limit: u32,
    cdn_base: &str,
) -> Result<Vec<SearchHelpChannel>, AppError> {
    let needle = format!("%{}%", normalize_search_needle(query));
    let limit = limit.clamp(1, 50);
    let rows = sqlx::query_as::<_, EntityIndexCard>(
        "SELECT DISTINCT c.id, c.slug, c.title AS display_name, c.thumb_path, c.video_count
         FROM channels c
         LEFT JOIN channel_aliases a ON a.channel_id = c.id
         WHERE c.title LIKE ? OR a.alias LIKE ? OR c.slug LIKE ?
         ORDER BY c.video_count DESC, c.title ASC
         LIMIT ?",
    )
    .bind(&needle)
    .bind(&needle)
    .bind(&needle)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let thumb = row.thumb_url(cdn_base);
            SearchHelpChannel {
                url: channel_profile_path(&row.slug),
                orig_name: row.display_name,
                thumb,
            }
        })
        .collect())
}

pub async fn count_videos_for_channel(
    pool: &MySqlPool,
    slug: &str,
    hd_only: bool,
) -> Result<u64, AppError> {
    let sql = if hd_only {
        "SELECT COUNT(DISTINCT v.id)
         FROM videos v
         INNER JOIN channels c ON c.id = v.channel_id
         WHERE c.slug = ? AND v.is_active = 1 AND v.is_hd = 1"
    } else {
        "SELECT COUNT(DISTINCT v.id)
         FROM videos v
         INNER JOIN channels c ON c.id = v.channel_id
         WHERE c.slug = ? AND v.is_active = 1"
    };
    let row: (i64,) = sqlx::query_as(sql).bind(slug).fetch_one(pool).await?;
    Ok(row.0.max(0) as u64)
}

pub async fn videos_for_channel(
    pool: &MySqlPool,
    slug: &str,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<EntityVideoThumb>, AppError> {
    let page = page.max(1);
    let per_page = per_page.clamp(1, 120);
    let offset = (page - 1) as i64 * per_page as i64;
    let order = entity_video_order_sql(sort);
    let sql = if hd_only {
        format!(
            "SELECT v.id AS video_id, v.slug, v.title, v.thumb_path, v.preview_path,
                v.duration_seconds, v.views, v.likes_pct, v.comments_count,
                v.published_at, v.wide_thumb
         FROM videos v
         INNER JOIN channels c ON c.id = v.channel_id
         WHERE c.slug = ? AND v.is_active = 1 AND v.is_hd = 1
         ORDER BY {order}
         LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT v.id AS video_id, v.slug, v.title, v.thumb_path, v.preview_path,
                v.duration_seconds, v.views, v.likes_pct, v.comments_count,
                v.published_at, v.wide_thumb
         FROM videos v
         INNER JOIN channels c ON c.id = v.channel_id
         WHERE c.slug = ? AND v.is_active = 1
         ORDER BY {order}
         LIMIT ? OFFSET ?"
        )
    };

    let rows = sqlx::query_as::<_, EntityVideoThumb>(&sql)
        .bind(slug)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

/// Expected MySQL DDL for local validation (sok-replica.3.1); kept as a single
/// string so handlers/migrations can reuse it without duplicating column lists.
pub const ENTITY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS pornstars (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    slug VARCHAR(191) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    thumb_path VARCHAR(512) NOT NULL DEFAULT '',
    banner_path VARCHAR(512) NULL,
    avatar_path VARCHAR(512) NULL,
    bio TEXT NULL,
    video_count INT UNSIGNED NOT NULL DEFAULT 0,
    verified TINYINT(1) NOT NULL DEFAULT 0,
    week_views BIGINT UNSIGNED NOT NULL DEFAULT 0,
    created_at DATETIME NULL,
    updated_at DATETIME NULL,
    UNIQUE KEY uq_pornstars_slug (slug),
    KEY idx_pornstars_week_views (week_views),
    KEY idx_pornstars_video_count (video_count)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS pornstar_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    pornstar_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(255) NOT NULL,
    UNIQUE KEY uq_pornstar_alias (pornstar_id, alias),
    KEY idx_pornstar_alias_lookup (alias),
    CONSTRAINT fk_pornstar_aliases_pornstar
        FOREIGN KEY (pornstar_id) REFERENCES pornstars (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS channels (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    slug VARCHAR(191) NOT NULL,
    title VARCHAR(255) NOT NULL,
    thumb_path VARCHAR(512) NOT NULL DEFAULT '',
    logo_path VARCHAR(512) NULL,
    banner_path VARCHAR(512) NULL,
    bio TEXT NULL,
    video_count INT UNSIGNED NOT NULL DEFAULT 0,
    network_name VARCHAR(255) NULL,
    week_views BIGINT UNSIGNED NOT NULL DEFAULT 0,
    created_at DATETIME NULL,
    updated_at DATETIME NULL,
    UNIQUE KEY uq_channels_slug (slug),
    KEY idx_channels_week_views (week_views),
    KEY idx_channels_video_count (video_count)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS channel_aliases (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    channel_id BIGINT UNSIGNED NOT NULL,
    alias VARCHAR(255) NOT NULL,
    UNIQUE KEY uq_channel_alias (channel_id, alias),
    KEY idx_channel_alias_lookup (alias),
    CONSTRAINT fk_channel_aliases_channel
        FOREIGN KEY (channel_id) REFERENCES channels (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS video_pornstars (
    video_id BIGINT UNSIGNED NOT NULL,
    pornstar_id BIGINT UNSIGNED NOT NULL,
    PRIMARY KEY (video_id, pornstar_id),
    KEY idx_video_pornstars_pornstar (pornstar_id),
    CONSTRAINT fk_video_pornstars_video
        FOREIGN KEY (video_id) REFERENCES videos (id) ON DELETE CASCADE,
    CONSTRAINT fk_video_pornstars_pornstar
        FOREIGN KEY (pornstar_id) REFERENCES pornstars (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
"#;

fn normalize_search_needle(input: &str) -> String {
    input.trim().replace('%', "").replace('_', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_url_joins_cdn_and_relative_path() {
        let url = media_url(DEFAULT_MEDIA_CDN, "fox-images/pornstars/angela-white.jpg");
        assert_eq!(
            url,
            "https://c.foxporn.tv/fox-images/pornstars/angela-white.jpg"
        );
    }

    #[test]
    fn profile_paths_match_site_routes() {
        assert_eq!(
            pornstar_profile_path("angela-white"),
            "/pornstar/angela-white"
        );
        assert_eq!(channel_profile_path("brazzers"), "/channel/brazzers");
    }

    #[test]
    fn entity_list_sort_parses_query_aliases() {
        assert_eq!(
            EntityListSort::from_query_param(Some("alpha")),
            EntityListSort::NameAsc
        );
        assert_eq!(
            EntityListSort::from_query_param(Some("video_count")),
            EntityListSort::VideoCountDesc
        );
        assert_eq!(
            EntityListSort::from_query_param(None),
            EntityListSort::Trending
        );
    }

    #[test]
    fn schema_reference_includes_core_tables() {
        assert!(ENTITY_SCHEMA_SQL.contains("CREATE TABLE IF NOT EXISTS pornstars"));
        assert!(ENTITY_SCHEMA_SQL.contains("CREATE TABLE IF NOT EXISTS channels"));
        assert!(ENTITY_SCHEMA_SQL.contains("video_pornstars"));
        assert!(ENTITY_SCHEMA_SQL.contains("pornstar_aliases"));
        assert!(ENTITY_SCHEMA_SQL.contains("channel_aliases"));
    }

    #[test]
    fn page_search_item_maps_from_index_card() {
        let card = EntityIndexCard {
            id: 1,
            slug: "angela-white".into(),
            display_name: "Angela White".into(),
            thumb_path: "fox-images/pornstars/angela-white.jpg".into(),
            video_count: 42,
        };
        let item: PornstarPageSearchItem = card.into();
        assert_eq!(item.url, "/pornstar/angela-white");
        assert_eq!(item.orig_name, "Angela White");
        assert_eq!(item.count_videos, 42);
    }
}
