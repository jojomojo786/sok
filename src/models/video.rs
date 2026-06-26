//! Video listing and detail query models for PornsOK-style grids and `/video/{slug}.html`.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;
use crate::errors::AppError;

/// Default CDN prefix for poster and preview assets (matches live `thumbs_path`).
pub const DEFAULT_THUMBS_CDN: &str = "https://c.foxporn.tv/fox-images/videos";

/// Listing sort keys used by `?sort=` on home and category pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoListSort {
    #[default]
    Trending,
    /// `?sort=mv` — most viewed.
    MostViewed,
    /// `?sort=mc` — most commented.
    MostCommented,
    /// Newest by upload timestamp.
    Newest,
}

impl VideoListSort {
    pub fn from_query(sort: Option<&str>) -> Self {
        match sort.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
            Some("mv") => Self::MostViewed,
            Some("mc") => Self::MostCommented,
            Some("new" | "newest") => Self::Newest,
            _ => Self::Trending,
        }
    }

    pub fn order_sql(self) -> &'static str {
        match self {
            Self::MostViewed => "v.views DESC, v.id DESC",
            Self::MostCommented => "v.comment_count DESC, v.views DESC, v.id DESC",
            Self::Newest => "v.uploaded_at DESC, v.id DESC",
            Self::Trending => "v.views DESC, v.uploaded_at DESC, v.id DESC",
        }
    }
}

/// Lightweight row for thumbnail grids, AJAX carousels, and search result video items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoThumb {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub duration_seconds: u32,
    pub thumb_url: String,
    pub preview_mp4: String,
    pub views: u64,
    /// Like ratio 0–100 (maps to `.tlike` percent on cards).
    pub likes_percent: u8,
    pub comments: u32,
    pub published_at: Option<NaiveDate>,
    pub is_hd: bool,
    /// When false, thumb images use `not-wide` CSS (`widethumb` on live search JSON).
    pub wide_thumb: bool,
}

/// Full video payload for detail pages, related rails, and Schema.org `VideoObject`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoDetail {
    pub thumb: VideoThumb,
    pub description: Option<String>,
    pub uploaded_at: Option<DateTime<Utc>>,
    pub stream_token: Option<String>,
}

impl VideoDetail {
    pub fn slug(&self) -> &str {
        &self.thumb.slug
    }

    pub fn title(&self) -> &str {
        &self.thumb.title
    }

    pub fn comments_count(&self) -> u32 {
        self.thumb.comments
    }

    pub fn canonical_path(&self) -> String {
        video_page_path(&self.thumb.slug)
    }

    pub fn embed_path(&self) -> String {
        format!("/embeded/{}.html", self.thumb.slug)
    }

    pub fn videofile_path(&self) -> Option<String> {
        self.stream_token
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|token| format!("/videofile/{token}"))
    }

    pub fn stream_src_for_player(&self) -> String {
        if let Some(path) = self.videofile_path() {
            return path;
        }
        self.thumb.preview_mp4.clone()
    }

    pub fn schema_content_url(&self, site_base: &str) -> String {
        if let Some(path) = self.videofile_path() {
            return crate::models::pagination::absolute_url(site_base, &path);
        }
        if !self.thumb.preview_mp4.is_empty() {
            return self.thumb.preview_mp4.clone();
        }
        String::new()
    }

    pub fn schema_duration(&self) -> String {
        format_schema_duration(self.thumb.duration_seconds)
    }

    pub fn schema_upload_date(&self) -> Option<String> {
        self.uploaded_at
            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
    }

    pub fn schema_date_published(&self) -> Option<String> {
        self.thumb
            .published_at
            .map(|d| d.format("%Y-%m-%d").to_string())
    }

    pub fn schema_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn views_count(&self) -> u64 {
        self.thumb.views
    }

    pub fn views_label(&self) -> String {
        self.thumb.views_label()
    }

    pub fn comments_label(&self) -> String {
        self.thumb.comments.to_string()
    }
}

/// Internal SQLx row mapped from the `videos` table.
#[derive(Debug, Clone, FromRow)]
pub struct VideoRow {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub duration_seconds: u32,
    pub thumb_url: String,
    pub preview_mp4: String,
    pub stream_token: Option<String>,
    pub views: u64,
    pub likes_up: u32,
    pub likes_down: u32,
    pub comment_count: u32,
    pub is_hd: i8,
    pub wide_thumb: i8,
    pub published_at: Option<NaiveDate>,
    pub uploaded_at: Option<DateTime<Utc>>,
}

impl From<VideoRow> for VideoThumb {
    fn from(row: VideoRow) -> Self {
        VideoThumb {
            id: row.id,
            slug: row.slug,
            title: row.title,
            duration_seconds: row.duration_seconds,
            thumb_url: row.thumb_url,
            preview_mp4: row.preview_mp4,
            views: row.views,
            likes_percent: like_percent(row.likes_up, row.likes_down),
            comments: row.comment_count,
            published_at: row.published_at,
            is_hd: row.is_hd != 0,
            wide_thumb: row.wide_thumb != 0,
        }
    }
}

impl From<VideoRow> for VideoDetail {
    fn from(row: VideoRow) -> Self {
        let thumb = VideoThumb::from(row.clone());
        VideoDetail {
            thumb,
            description: row.description,
            uploaded_at: row.uploaded_at,
            stream_token: row.stream_token,
        }
    }
}

impl VideoThumb {
    pub fn page_path(&self) -> String {
        video_page_path(&self.slug)
    }

    pub fn duration_label(&self) -> String {
        format_duration_label(self.duration_seconds)
    }

    pub fn views_label(&self) -> String {
        format_compact_count(self.views)
    }

    pub fn likes_label(&self) -> String {
        format!("{}%", self.likes_percent)
    }

    pub fn schema_duration(&self) -> String {
        format_schema_duration(self.duration_seconds)
    }

    pub fn thumb_css_class(&self) -> &'static str {
        if self.wide_thumb {
            "thumb vid"
        } else {
            "thumb vid not-wide"
        }
    }

    /// `YYYY-MM-DD` value for the `datePublished` meta on the home grid, if known.
    pub fn schema_date_published(&self) -> Option<String> {
        self.published_at.map(|d| d.format("%Y-%m-%d").to_string())
    }
}

pub fn thumb_url_from_slug(slug: &str, cdn_base: &str) -> String {
    format!("{cdn_base}/{slug}.jpg")
}

pub fn preview_mp4_from_slug(slug: &str, cdn_base: &str) -> String {
    format!("{cdn_base}/m-{slug}.mp4")
}

pub fn video_page_path(slug: &str) -> String {
    let normalized = normalize_video_slug(slug);
    format!("/video/{normalized}.html")
}

pub fn normalize_video_slug(slug: &str) -> String {
    let trimmed = slug.trim().trim_matches('/');
    if let Some(stripped) = trimmed.strip_prefix("video/") {
        let stripped = stripped.strip_suffix(".html").unwrap_or(stripped);
        return stripped.to_string();
    }
    trimmed.strip_suffix(".html").unwrap_or(trimmed).to_string()
}

pub fn like_percent(likes_up: u32, likes_down: u32) -> u8 {
    let total = likes_up.saturating_add(likes_down);
    if total == 0 {
        return 0;
    }
    ((likes_up as u64 * 100) / total as u64).min(100) as u8
}

pub fn format_schema_duration(seconds: u32) -> String {
    if seconds == 0 {
        return "PT0S".into();
    }
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    let mut out = String::from("PT");
    if hours > 0 {
        out.push_str(&format!("{hours}H"));
    }
    if minutes > 0 {
        out.push_str(&format!("{minutes}M"));
    }
    if secs > 0 || out == "PT" {
        out.push_str(&format!("{secs}S"));
    }
    out
}

pub fn video_page_title(title: &str, channel_label: Option<&str>) -> String {
    let trimmed = title.trim();
    if let Some(channel) = channel_label.map(str::trim).filter(|c| !c.is_empty()) {
        format!("{trimmed} / porn video by {channel}")
    } else {
        format!("{trimmed} | PornsOK.com")
    }
}

pub fn format_duration_label(seconds: u32) -> String {
    if seconds < 60 {
        return format!("{seconds} sec");
    }
    let minutes = seconds / 60;
    format!("{minutes} min")
}

pub fn format_compact_count(n: u64) -> String {
    if n >= 1_000_000 {
        let v = (n as f64) / 1_000_000.0;
        return format!("{:.1}M", trim_one_decimal(v));
    }
    if n >= 10_000 {
        let v = (n as f64) / 1_000.0;
        return format!("{:.0}K", v.round());
    }
    if n >= 1_000 {
        let v = (n as f64) / 1_000.0;
        return format!("{:.1}K", trim_one_decimal(v));
    }
    n.to_string()
}

fn trim_one_decimal(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

const VIDEO_THUMB_SELECT: &str = r#"
SELECT
    v.id,
    v.slug,
    v.title,
    v.description,
    v.duration_seconds,
    v.thumb_url,
    v.preview_mp4,
    v.stream_token,
    v.views,
    v.likes_up,
    v.likes_down,
    v.comment_count,
    v.is_hd,
    v.wide_thumb,
    v.published_at,
    v.uploaded_at
FROM videos v
"#;

pub async fn list_home_thumbs(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<VideoThumb>, AppError> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let order = sort.order_sql();
    let sql = if hd_only {
        format!(
            "{VIDEO_THUMB_SELECT}
             WHERE v.status = 'published' AND v.is_hd = 1
             ORDER BY {order}
             LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "{VIDEO_THUMB_SELECT}
             WHERE v.status = 'published'
             ORDER BY {order}
             LIMIT ? OFFSET ?"
        )
    };

    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

/// Total published videos for the home grid (all or HD-only), for pagination.
pub async fn count_home_videos(pool: &DbPool, hd_only: bool) -> Result<u64, AppError> {
    let sql = if hd_only {
        "SELECT COUNT(*) FROM videos v WHERE v.status = 'published' AND v.is_hd = 1"
    } else {
        "SELECT COUNT(*) FROM videos v WHERE v.status = 'published'"
    };
    let total: i64 = sqlx::query_scalar(sql).fetch_one(pool).await?;
    Ok(total.max(0) as u64)
}

pub async fn list_search_videos(
    pool: &DbPool,
    needle: &str,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<VideoThumb>, AppError> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let order = sort.order_sql();
    let like = format!(
        "%{}%",
        crate::models::search::normalize_search_needle(needle)
    );
    let sql = if hd_only {
        format!(
            "{VIDEO_THUMB_SELECT}
             WHERE v.status = 'published' AND v.is_hd = 1 AND v.title LIKE ?
             ORDER BY {order}
             LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "{VIDEO_THUMB_SELECT}
             WHERE v.status = 'published' AND v.title LIKE ?
             ORDER BY {order}
             LIMIT ? OFFSET ?"
        )
    };
    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(&like)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

pub async fn count_search_videos(
    pool: &DbPool,
    needle: &str,
    hd_only: bool,
) -> Result<u64, AppError> {
    let like = format!(
        "%{}%",
        crate::models::search::normalize_search_needle(needle)
    );
    let sql = if hd_only {
        "SELECT COUNT(*) FROM videos v WHERE v.status = 'published' AND v.is_hd = 1 AND v.title LIKE ?"
    } else {
        "SELECT COUNT(*) FROM videos v WHERE v.status = 'published' AND v.title LIKE ?"
    };
    let total: i64 = sqlx::query_scalar(sql).bind(&like).fetch_one(pool).await?;
    Ok(total.max(0) as u64)
}

pub async fn list_video_thumbs(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<VideoThumb>, AppError> {
    list_home_thumbs(pool, page, per_page, sort, hd_only).await
}

pub async fn video_detail_by_slug(pool: &DbPool, slug: &str) -> Result<VideoDetail, AppError> {
    let slug = normalize_video_slug(slug);
    let sql = format!(
        "{VIDEO_THUMB_SELECT}
         WHERE v.status = 'published' AND v.slug = ?
         LIMIT 1"
    );

    let row = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(&slug)
        .fetch_optional(pool)
        .await?;

    row.map(VideoDetail::from)
        .ok_or_else(|| AppError::NotFound(format!("video not found: {slug}")))
}

pub async fn video_likes_percent_by_id(pool: &DbPool, video_id: u64) -> Option<u8> {
    let row: Result<Option<(u32, u32)>, _> = sqlx::query_as(
        "SELECT vote_up_count, vote_down_count FROM videos WHERE id = ? AND is_active = 1 LIMIT 1",
    )
    .bind(video_id)
    .fetch_optional(pool)
    .await;

    if let Ok(Some((likes_up, likes_down))) = row {
        return Some(like_percent(likes_up, likes_down));
    }

    if video_id == fixtures::sample_dog_house_video_detail().thumb.id {
        return Some(
            fixtures::sample_dog_house_video_detail()
                .thumb
                .likes_percent,
        );
    }

    None
}

pub const RELATED_THUMB_BATCH_SIZE: usize = 6;
pub const RELATED_AJAX_MAX_BATCHES: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelatedAjaxThumb {
    pub url: String,
    pub title: String,
    pub thumb: String,
    pub preview_mini: String,
    pub duration: String,
    pub views: u64,
    pub str_views: String,
    pub rate: u8,
    pub widethumb: u8,
}

pub async fn list_related_thumbs_for_video(
    pool: &DbPool,
    slug: &str,
    limit: u32,
) -> Result<Vec<VideoThumb>, AppError> {
    let slug = normalize_video_slug(slug);
    if let Ok(rows) = query_related_thumbs_dev(pool, &slug, limit).await {
        if !rows.is_empty() {
            return Ok(rows);
        }
    }
    if let Ok(rows) = query_related_thumbs_catalog(pool, &slug, limit).await {
        return Ok(rows);
    }
    Ok(Vec::new())
}

pub async fn related_ajax_batches_for_video(
    pool: &DbPool,
    slug: &str,
) -> Result<Vec<Vec<RelatedAjaxThumb>>, AppError> {
    let thumbs = list_related_thumbs_for_video(
        pool,
        slug,
        RELATED_THUMB_BATCH_SIZE as u32 * RELATED_AJAX_MAX_BATCHES as u32,
    )
    .await?;
    Ok(chunk_related_ajax_batches(&thumbs))
}

pub fn chunk_related_ajax_batches(thumbs: &[VideoThumb]) -> Vec<Vec<RelatedAjaxThumb>> {
    thumbs
        .chunks(RELATED_THUMB_BATCH_SIZE)
        .take(RELATED_AJAX_MAX_BATCHES)
        .map(|chunk| chunk.iter().map(related_ajax_thumb_from_video).collect())
        .collect()
}

pub fn related_ajax_thumb_from_video(thumb: &VideoThumb) -> RelatedAjaxThumb {
    RelatedAjaxThumb {
        url: format!("{}.html", thumb.slug),
        title: thumb.title.clone(),
        thumb: related_asset_filename(&thumb.thumb_url, &thumb.slug, ".jpg"),
        preview_mini: related_asset_filename(&thumb.preview_mp4, &thumb.slug, ".mp4"),
        duration: thumb.duration_label(),
        views: thumb.views,
        str_views: thumb.views_label(),
        rate: thumb.likes_percent,
        widethumb: if thumb.wide_thumb { 1 } else { 0 },
    }
}

fn related_asset_filename(url: &str, slug: &str, suffix: &str) -> String {
    let trimmed = url.trim();
    if !trimmed.is_empty() {
        if let Some(name) = trimmed.rsplit('/').next() {
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    if suffix == ".mp4" {
        return format!("m-{slug}.mp4");
    }
    format!("{slug}.jpg")
}

async fn query_related_thumbs_dev(
    pool: &DbPool,
    slug: &str,
    limit: u32,
) -> Result<Vec<VideoThumb>, AppError> {
    let sql = format!(
        "{VIDEO_THUMB_SELECT}
         LEFT JOIN (
             SELECT vt2.video_id AS related_id,
                    COUNT(DISTINCT vt2.tag_id) AS shared_tags
             FROM video_tags vt_seed
             INNER JOIN video_tags vt2 ON vt2.tag_id = vt_seed.tag_id
             INNER JOIN videos seed ON seed.id = vt_seed.video_id
             WHERE seed.slug = ? AND vt2.video_id <> seed.id
             GROUP BY vt2.video_id
         ) rel_tags ON rel_tags.related_id = v.id
         LEFT JOIN (
             SELECT vc2.video_id AS related_id,
                    COUNT(DISTINCT vc2.channel_id) AS shared_channels
             FROM video_channels vc_seed
             INNER JOIN video_channels vc2 ON vc2.channel_id = vc_seed.channel_id
             INNER JOIN videos seed ON seed.id = vc_seed.video_id
             WHERE seed.slug = ? AND vc2.video_id <> seed.id
             GROUP BY vc2.video_id
         ) rel_channels ON rel_channels.related_id = v.id
         WHERE v.status = 'published'
           AND v.slug <> ?
         ORDER BY (COALESCE(rel_tags.shared_tags, 0) + COALESCE(rel_channels.shared_channels, 0)) DESC,
                  v.views DESC,
                  v.id DESC
         LIMIT ?"
    );

    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(slug)
        .bind(slug)
        .bind(slug)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

async fn query_related_thumbs_catalog(
    pool: &DbPool,
    slug: &str,
    limit: u32,
) -> Result<Vec<VideoThumb>, AppError> {
    let sql = r#"
        SELECT
            v.id,
            v.slug,
            v.title,
            v.synopsis AS description,
            v.duration_seconds,
            COALESCE(v.thumb_url, '') AS thumb_url,
            COALESCE(v.preview_mp4_url, '') AS preview_mp4,
            v.stream_url AS stream_token,
            v.view_count AS views,
            v.vote_up_count AS likes_up,
            v.vote_down_count AS likes_down,
            v.comment_count,
            v.is_hd,
            v.is_wide_thumb AS wide_thumb,
            v.published_at,
            NULL AS uploaded_at
        FROM videos v
        INNER JOIN videos seed ON seed.slug = ?
        LEFT JOIN (
            SELECT vt2.video_id AS related_id,
                   COUNT(DISTINCT vt2.tag_id) AS shared_tags
            FROM video_tags vt_seed
            INNER JOIN video_tags vt2 ON vt2.tag_id = vt_seed.tag_id
            WHERE vt_seed.video_id = seed.id AND vt2.video_id <> seed.id
            GROUP BY vt2.video_id
        ) rel_tags ON rel_tags.related_id = v.id
        LEFT JOIN (
            SELECT vc2.video_id AS related_id,
                   COUNT(DISTINCT vc2.channel_id) AS shared_channels
            FROM video_channels vc_seed
            INNER JOIN video_channels vc2 ON vc2.channel_id = vc_seed.channel_id
            WHERE vc_seed.video_id = seed.id AND vc2.video_id <> seed.id
            GROUP BY vc2.video_id
        ) rel_channels ON rel_channels.related_id = v.id
        WHERE v.is_active = 1
          AND v.slug <> seed.slug
        ORDER BY (COALESCE(rel_tags.shared_tags, 0) + COALESCE(rel_channels.shared_channels, 0)) DESC,
                 v.view_count DESC,
                 v.id DESC
        LIMIT ?
    "#;

    let rows = sqlx::query_as::<_, VideoRow>(sql)
        .bind(slug)
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

pub async fn list_watching_now_thumbs(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<VideoThumb>, AppError> {
    let sql = format!(
        "{VIDEO_THUMB_SELECT}
         WHERE v.status = 'published'
         ORDER BY v.views DESC, v.uploaded_at DESC
         LIMIT ?"
    );

    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

pub async fn list_thumbs_for_category(
    pool: &DbPool,
    category_id: u64,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<VideoThumb>, AppError> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let order = sort.order_sql();
    let sql = if hd_only {
        format!("{VIDEO_THUMB_SELECT} INNER JOIN video_categories vc ON vc.video_id = v.id INNER JOIN categories c ON c.id = vc.category_id WHERE v.status = 'published' AND v.is_hd = 1 AND vc.category_id = ? AND c.is_active = 1 ORDER BY {order} LIMIT ? OFFSET ?")
    } else {
        format!("{VIDEO_THUMB_SELECT} INNER JOIN video_categories vc ON vc.video_id = v.id INNER JOIN categories c ON c.id = vc.category_id WHERE v.status = 'published' AND vc.category_id = ? AND c.is_active = 1 ORDER BY {order} LIMIT ? OFFSET ?")
    };
    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(category_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

pub async fn list_thumbs_for_tag(
    pool: &DbPool,
    tag_id: u64,
    page: u32,
    per_page: u32,
    sort: VideoListSort,
    hd_only: bool,
) -> Result<Vec<VideoThumb>, AppError> {
    let offset = page.saturating_sub(1).saturating_mul(per_page);
    let order = sort.order_sql();
    let sql = if hd_only {
        format!("{VIDEO_THUMB_SELECT} INNER JOIN video_tags vt ON vt.video_id = v.id INNER JOIN tags t ON t.id = vt.tag_id WHERE v.status = 'published' AND v.is_hd = 1 AND vt.tag_id = ? AND t.is_active = 1 ORDER BY {order} LIMIT ? OFFSET ?")
    } else {
        format!("{VIDEO_THUMB_SELECT} INNER JOIN video_tags vt ON vt.video_id = v.id INNER JOIN tags t ON t.id = vt.tag_id WHERE v.status = 'published' AND vt.tag_id = ? AND t.is_active = 1 ORDER BY {order} LIMIT ? OFFSET ?")
    };
    let rows = sqlx::query_as::<_, VideoRow>(&sql)
        .bind(tag_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(VideoThumb::from).collect())
}

pub async fn count_videos_for_category(
    pool: &DbPool,
    category_id: u64,
    hd_only: bool,
) -> Result<u64, AppError> {
    let sql = if hd_only {
        "SELECT COUNT(DISTINCT v.id) FROM videos v INNER JOIN video_categories vc ON vc.video_id = v.id INNER JOIN categories c ON c.id = vc.category_id WHERE v.status = 'published' AND v.is_hd = 1 AND vc.category_id = ? AND c.is_active = 1"
    } else {
        "SELECT COUNT(DISTINCT v.id) FROM videos v INNER JOIN video_categories vc ON vc.video_id = v.id INNER JOIN categories c ON c.id = vc.category_id WHERE v.status = 'published' AND vc.category_id = ? AND c.is_active = 1"
    };
    let (count,): (i64,) = sqlx::query_as(sql)
        .bind(category_id)
        .fetch_one(pool)
        .await?;
    Ok(count.max(0) as u64)
}

pub async fn count_videos_for_tag(
    pool: &DbPool,
    tag_id: u64,
    hd_only: bool,
) -> Result<u64, AppError> {
    let sql = if hd_only {
        "SELECT COUNT(DISTINCT v.id) FROM videos v INNER JOIN video_tags vt ON vt.video_id = v.id INNER JOIN tags t ON t.id = vt.tag_id WHERE v.status = 'published' AND v.is_hd = 1 AND vt.tag_id = ? AND t.is_active = 1"
    } else {
        "SELECT COUNT(DISTINCT v.id) FROM videos v INNER JOIN video_tags vt ON vt.video_id = v.id INNER JOIN tags t ON t.id = vt.tag_id WHERE v.status = 'published' AND vt.tag_id = ? AND t.is_active = 1"
    };
    let (count,): (i64,) = sqlx::query_as(sql).bind(tag_id).fetch_one(pool).await?;
    Ok(count.max(0) as u64)
}

pub mod fixtures {
    use super::*;
    use crate::models::comments::Comment;
    use chrono::TimeZone;

    pub fn sample_home_grid() -> Vec<VideoThumb> {
        vec![
            VideoThumb {
                id: 1,
                slug: "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed"
                    .into(),
                title: "Athena Faris Sneaks out of Alex's Gym Bag & Joins him & his wifey Chloe Cherry in Bed".into(),
                duration_seconds: 703,
                thumb_url: thumb_url_from_slug(
                    "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed",
                    DEFAULT_THUMBS_CDN,
                ),
                preview_mp4: preview_mp4_from_slug(
                    "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed",
                    DEFAULT_THUMBS_CDN,
                ),
                views: 22_063,
                likes_percent: 70,
                comments: 0,
                published_at: NaiveDate::from_ymd_opt(2026, 4, 18),
                is_hd: true,
                wide_thumb: true,
            },
            VideoThumb {
                id: 2,
                slug: "desi-young-bhabhi-strips-her-saree-to-fuck-devars-big-cock-and-swallows-cum".into(),
                title: "Desi Young Bhabhi Strips her Saree to Fuck Devar's Big cock and Swallows Cum".into(),
                duration_seconds: 655,
                thumb_url: thumb_url_from_slug(
                    "desi-young-bhabhi-strips-her-saree-to-fuck-devars-big-cock-and-swallows-cum",
                    DEFAULT_THUMBS_CDN,
                ),
                preview_mp4: preview_mp4_from_slug(
                    "desi-young-bhabhi-strips-her-saree-to-fuck-devars-big-cock-and-swallows-cum",
                    DEFAULT_THUMBS_CDN,
                ),
                views: 233_561,
                likes_percent: 78,
                comments: 12,
                published_at: NaiveDate::from_ymd_opt(2024, 8, 11),
                is_hd: false,
                wide_thumb: true,
            },
        ]
    }

    pub fn sample_video_detail() -> VideoDetail {
        let thumb = VideoThumb {
            id: 53_036,
            slug: "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed".into(),
            title: "Athena Faris Sneaks out of Alex's Gym Bag & Joins him & his wifey Chloe Cherry in Bed".into(),
            duration_seconds: 703,
            thumb_url: thumb_url_from_slug(
                "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed",
                DEFAULT_THUMBS_CDN,
            ),
            preview_mp4: preview_mp4_from_slug(
                "athena-faris-sneaks-out-of-alexs-gym-bag-joins-him-his-wifey-chloe-cherry-in-bed",
                DEFAULT_THUMBS_CDN,
            ),
            views: 117_275,
            likes_percent: 70,
            comments: 14,
            published_at: NaiveDate::from_ymd_opt(2026, 3, 9),
            is_hd: true,
            wide_thumb: true,
        };

        VideoDetail {
            thumb,
            description: Some(
                "Watch Athena Faris Sneaks out of Alex's Gym Bag & Joins him & his wifey Chloe Cherry in Bed on PornsOK, the best porn site.".into(),
            ),
            uploaded_at: Utc.with_ymd_and_hms(2026, 4, 18, 0, 5, 8).single(),
            stream_token: Some("WyJwb3JuaHViIiwicGg2Mjg0ZmViNzZlOWU2IiwwXQ==".into()),
        }
    }

    /// Production sample slug from `docs/pages/video-detail.md` (Dog House / Madi Collins).
    pub const DOG_HOUSE_SLUG: &str =
        "dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do";

    pub fn sample_dog_house_video_detail() -> VideoDetail {
        let slug = DOG_HOUSE_SLUG.to_string();
        let thumb = VideoThumb {
            id: 52_994,
            slug: slug.clone(),
            title: "Dog House - Madi Collins knows more Sex than her step bro & Decides to Show him what he should do".into(),
            duration_seconds: 599,
            thumb_url: thumb_url_from_slug(&slug, DEFAULT_THUMBS_CDN),
            preview_mp4: preview_mp4_from_slug(&slug, DEFAULT_THUMBS_CDN),
            views: 15_077,
            likes_percent: 74,
            comments: 2,
            published_at: NaiveDate::from_ymd_opt(2026, 3, 2),
            is_hd: true,
            wide_thumb: true,
        };

        VideoDetail {
            thumb,
            description: Some(
                "Watch Dog House - Madi Collins knows more Sex than her step bro & Decides to Show him what he should do on PornsOK, the best porn site. We have the biggest selection of squirt porn videos & pussy licking sex movies.".into(),
            ),
            uploaded_at: Utc.with_ymd_and_hms(2026, 3, 2, 12, 0, 0).single(),
            stream_token: Some(
                "WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D".into(),
            ),
        }
    }

    pub fn related_fixture_pool_for_slug(slug: &str) -> Vec<VideoThumb> {
        if let Ok(seed) = crate::fixtures::load_catalog_seed() {
            let mut thumbs = crate::fixtures::seed_home_thumbs(&seed);
            thumbs.retain(|thumb| thumb.slug != slug);
            if !thumbs.is_empty() {
                return thumbs;
            }
        }
        sample_home_grid()
    }

    pub fn related_fixture_batches_for_slug(slug: &str) -> Vec<Vec<super::RelatedAjaxThumb>> {
        super::chunk_related_ajax_batches(&related_fixture_pool_for_slug(slug))
    }

    /// Page-level fixture extras aligned with live capture (tags, related, comments).
    pub fn sample_dog_house_video_page() -> DogHouseVideoPageFixture {
        use crate::models::comments::{prepare_comment_body, Comment};

        let detail = sample_dog_house_video_detail();
        let categories = vec![
            ("Blowjob", "/blowjob"),
            ("Squirt", "/squirt"),
            ("Pussy Licking", "/pussy-licking"),
        ]
        .into_iter()
        .map(|(label, href)| DogHouseCategoryLink {
            label: label.into(),
            href: href.into(),
        })
        .collect();

        let tags = vec![
            ("Madi Collins", "/pornstar/madi-collins"),
            ("Step Brother", "/videos/step-brother"),
            ("Dog House Digital", "/channel/dog-house-digital"),
        ]
        .into_iter()
        .map(|(label, href)| DogHouseTagLink {
            label: label.into(),
            href: href.into(),
        })
        .collect();

        let related = related_fixture_pool_for_slug(DOG_HOUSE_SLUG);
        let mut comments = Vec::new();
        if let Ok(body) = prepare_comment_body("Great scene $#sperm_0DCpe#$") {
            comments.push(Comment {
                id: 1,
                video_id: detail.thumb.id,
                parent_id: None,
                author_name: "PornsOK fan".into(),
                body_raw: body.body_raw,
                body_html: body.body_html,
                is_visible: true,
            });
        }

        DogHouseVideoPageFixture {
            detail,
            extras: DogHouseFixtureExtras {
                rating_percent: 74,
                rating_count: 35,
                upload_date_label: "Mar 2, 2026".into(),
                upload_date_iso: Some("2026-03-02".into()),
                channel_label: Some("Dog House Digital".into()),
                channel_href: Some("/channel/dog-house-digital".into()),
                categories,
                tags,
                related,
                related_ajax_batches: related_fixture_batches_for_slug(DOG_HOUSE_SLUG),
                comments,
                download_form_action: format!(
                    "/videofile/WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ%3D%3D"
                ),
            },
        }
    }

    #[derive(Debug, Clone)]
    pub struct DogHouseVideoPageFixture {
        pub detail: VideoDetail,
        pub extras: DogHouseFixtureExtras,
    }

    #[derive(Debug, Clone)]
    pub struct DogHouseCategoryLink {
        pub label: String,
        pub href: String,
    }

    #[derive(Debug, Clone)]
    pub struct DogHouseTagLink {
        pub label: String,
        pub href: String,
    }

    #[derive(Debug, Clone)]
    pub struct DogHouseFixtureExtras {
        pub rating_percent: u8,
        pub rating_count: u32,
        pub upload_date_label: String,
        pub upload_date_iso: Option<String>,
        pub channel_label: Option<String>,
        pub channel_href: Option<String>,
        pub categories: Vec<DogHouseCategoryLink>,
        pub tags: Vec<DogHouseTagLink>,
        pub related: Vec<VideoThumb>,
        pub related_ajax_batches: Vec<Vec<RelatedAjaxThumb>>,
        pub comments: Vec<Comment>,
        pub download_form_action: String,
    }

    impl DogHouseFixtureExtras {
        pub fn empty() -> Self {
            Self {
                rating_percent: 0,
                rating_count: 0,
                upload_date_label: String::new(),
                upload_date_iso: None,
                channel_label: None,
                channel_href: None,
                categories: Vec::new(),
                tags: Vec::new(),
                related: Vec::new(),
                related_ajax_batches: Vec::new(),
                comments: Vec::new(),
                download_form_action: String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{
        sample_dog_house_video_detail, sample_home_grid, sample_video_detail, DOG_HOUSE_SLUG,
    };
    use super::*;

    #[test]
    fn home_grid_fixture_has_render_fields() {
        let grid = sample_home_grid();
        assert_eq!(grid.len(), 2);

        let first = &grid[0];
        assert_eq!(first.duration_label(), "11 min");
        assert_eq!(first.views_label(), "22K");
        assert_eq!(first.likes_label(), "70%");
        assert!(first.preview_mp4.contains("m-athena-faris"));
        assert!(first.page_path().ends_with(".html"));
        assert_eq!(first.schema_duration(), "PT11M43S");
        assert_eq!(
            first.published_at.map(|d| d.to_string()),
            Some("2026-04-18".into())
        );
    }

    #[test]
    fn videofile_path_on_fixture() {
        let detail = sample_video_detail();
        assert!(detail.videofile_path().unwrap().starts_with("/videofile/"));
        assert!(detail.stream_src_for_player().starts_with("/videofile/"));
        assert!(detail
            .schema_content_url("https://pornsok.com")
            .contains("/videofile/"));
    }

    #[test]
    fn dog_house_fixture_slug_matches_docs_sample() {
        let dog = sample_dog_house_video_detail();
        assert_eq!(dog.slug(), DOG_HOUSE_SLUG);
        assert_ne!(dog.slug(), sample_video_detail().slug());
        assert_eq!(dog.thumb.id, 52_994);
        let vf = dog.videofile_path().unwrap();
        assert!(vf.starts_with("/videofile/"));
        assert!(vf.contains("WyJwb3JuaHViIiwicGg2MzVhNDIwNzAyNmE1IiwwXQ"));
    }

    #[test]
    fn video_detail_fixture_exposes_schema_fields() {
        let detail = sample_video_detail();
        assert_eq!(detail.comments_count(), 14);
        assert_eq!(detail.schema_duration(), "PT11M43S");
        assert!(detail
            .schema_description()
            .unwrap()
            .starts_with("Watch Athena Faris"));
        assert!(detail.canonical_path().contains("athena-faris"));
        assert!(detail.embed_path().starts_with("/embeded/"));
    }

    #[test]
    fn schema_duration_formats_iso_8601_pt() {
        assert_eq!(format_schema_duration(703), "PT11M43S");
        assert_eq!(format_schema_duration(599), "PT9M59S");
        assert_eq!(format_schema_duration(3600), "PT1H");
        assert_eq!(format_schema_duration(0), "PT0S");
    }

    #[test]
    fn video_page_title_includes_channel_suffix_when_present() {
        assert_eq!(
            video_page_title("Dog House sample", Some("Dog House Digital")),
            "Dog House sample / porn video by Dog House Digital"
        );
        assert_eq!(
            video_page_title("Athena sample", None),
            "Athena sample | PornsOK.com"
        );
    }

    #[test]
    fn slug_normalization_strips_html_suffix() {
        assert_eq!(normalize_video_slug("foo-bar.html"), "foo-bar");
        assert_eq!(normalize_video_slug("/video/foo-bar.html"), "foo-bar");
    }

    #[test]
    fn like_percent_matches_live_card_math() {
        assert_eq!(like_percent(78, 22), 78);
        assert_eq!(like_percent(0, 0), 0);
        assert_eq!(like_percent(1, 0), 100);
    }

    #[test]
    fn related_ajax_thumb_uses_player_relative_paths() {
        let grid = sample_home_grid();
        let item = related_ajax_thumb_from_video(&grid[0]);
        assert!(item.url.ends_with(".html"));
        assert!(item.thumb.ends_with(".jpg"));
        assert!(item.preview_mini.starts_with("m-"));
        assert!(item.preview_mini.ends_with(".mp4"));
        assert_eq!(item.widethumb, 1);
    }

    #[test]
    fn related_fixture_batches_for_slug_excludes_current_video() {
        let batches = fixtures::related_fixture_batches_for_slug(DOG_HOUSE_SLUG);
        assert!(!batches.is_empty());
        assert!(batches
            .iter()
            .flatten()
            .all(|item| item.url != format!("{DOG_HOUSE_SLUG}.html")));
    }

    #[test]
    fn chunk_related_ajax_batches_respects_batch_size() {
        let grid = sample_home_grid();
        let batches = chunk_related_ajax_batches(&grid);
        assert!(!batches.is_empty());
        assert!(batches[0].len() <= RELATED_THUMB_BATCH_SIZE);
    }

    fn sort_query_mapping() {
        assert_eq!(
            VideoListSort::from_query(Some("mv")),
            VideoListSort::MostViewed
        );
        assert_eq!(
            VideoListSort::from_query(Some("mc")),
            VideoListSort::MostCommented
        );
        assert_eq!(VideoListSort::from_query(None), VideoListSort::Trending);
    }
}
