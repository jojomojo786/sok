//! Production-shaped JSON for POST `/ajax/search_help` (header autocomplete).
//!
//! Mirrors the live response in `docs/raw/search_help.body`: a top-level
//! `search_text` echo plus `pornstars`, `channels`, and `videos` groups whose
//! item keys match what `static/js/main.min.js` consumes.
//!
//! The handler uses the live MySQL catalog as the primary source and falls back
//! to the bundled fixtures (`fixtures/catalog_seed.json`) when the database is
//! unavailable or empty.

use serde::Serialize;

use crate::db::DbPool;
use crate::errors::AppError;
use crate::fixtures::{CatalogSeed, SeedChannel, SeedPornstar, SeedVideo};
use crate::models::entities::{
    channel_profile_path, media_url, pornstar_profile_path, DEFAULT_MEDIA_CDN,
};
use crate::models::video::video_page_path;

/// Maximum suggestions returned per group (pornstars/channels/videos).
pub const SEARCH_HELP_GROUP_LIMIT: u32 = 6;

/// Pornstar suggestion. Keys match production:
/// `url_pornstar`, `name`, `orig_name`, `thumb`, `count_videos`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHelpPornstarItem {
    pub url_pornstar: String,
    pub name: String,
    pub orig_name: String,
    pub thumb: String,
    pub count_videos: String,
}

/// Channel suggestion. Keys match production:
/// `url`, `orig_name`, `rus_name`, `thumb`, `count_videos`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHelpChannelItem {
    pub url: String,
    pub orig_name: String,
    pub rus_name: String,
    pub thumb: String,
    pub count_videos: String,
}

/// Video suggestion. Keys match production: `url`, `title`, `thumb`, `widethumb`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHelpVideoItem {
    pub url: String,
    pub title: String,
    pub thumb: String,
    pub widethumb: String,
}

/// Full `/ajax/search_help` response. Field order matches the live body
/// (`pornstars`, `channels`, `videos`, `search_text`), though clients key by
/// name so ordering is informational only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SearchHelpResponse {
    pub pornstars: Vec<SearchHelpPornstarItem>,
    pub channels: Vec<SearchHelpChannelItem>,
    pub videos: Vec<SearchHelpVideoItem>,
    pub search_text: String,
}

impl SearchHelpResponse {
    /// True when every group is empty (used to trigger fixture fallback).
    pub fn is_empty(&self) -> bool {
        self.pornstars.is_empty() && self.channels.is_empty() && self.videos.is_empty()
    }
}

#[derive(Debug, sqlx::FromRow)]
struct PornstarHelpRow {
    slug: String,
    display_name: String,
    thumb_path: String,
    video_count: u32,
}

#[derive(Debug, sqlx::FromRow)]
struct ChannelHelpRow {
    slug: String,
    title: String,
    thumb_path: String,
    video_count: u32,
}

#[derive(Debug, sqlx::FromRow)]
struct VideoHelpRow {
    slug: String,
    title: String,
    thumb_url: String,
    wide_thumb: i8,
}

fn normalize_needle(input: &str) -> String {
    input.trim().replace('%', "").replace('_', "")
}

fn pornstar_thumb(path: &str, slug: &str) -> String {
    if path.trim().is_empty() {
        crate::models::entities::pornstar_thumb_url(DEFAULT_MEDIA_CDN, slug)
    } else {
        media_url(DEFAULT_MEDIA_CDN, path)
    }
}

fn channel_thumb(path: &str, slug: &str) -> String {
    if path.trim().is_empty() {
        crate::models::entities::channel_thumb_url(DEFAULT_MEDIA_CDN, slug)
    } else {
        media_url(DEFAULT_MEDIA_CDN, path)
    }
}

/// Build the live response from MySQL. Empty queries return the most popular
/// suggestions (matching production's non-empty default body for `text=`).
pub async fn search_help_from_db(
    pool: &DbPool,
    text: &str,
    limit: u32,
) -> Result<SearchHelpResponse, AppError> {
    let trimmed = text.trim();
    let limit = limit.clamp(1, 50);

    let pornstars = if trimmed.is_empty() {
        sqlx::query_as::<_, PornstarHelpRow>(
            "SELECT slug, display_name, thumb_path, video_count
             FROM pornstars
             ORDER BY week_views DESC, video_count DESC, display_name ASC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        let needle = format!("%{}%", normalize_needle(trimmed));
        sqlx::query_as::<_, PornstarHelpRow>(
            "SELECT DISTINCT p.slug, p.display_name, p.thumb_path, p.video_count
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
        .await?
    };

    let channels = if trimmed.is_empty() {
        sqlx::query_as::<_, ChannelHelpRow>(
            "SELECT slug, title, thumb_path, video_count
             FROM channels
             ORDER BY week_views DESC, video_count DESC, title ASC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        let needle = format!("%{}%", normalize_needle(trimmed));
        sqlx::query_as::<_, ChannelHelpRow>(
            "SELECT DISTINCT c.slug, c.title, c.thumb_path, c.video_count
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
        .await?
    };

    let videos = if trimmed.is_empty() {
        sqlx::query_as::<_, VideoHelpRow>(
            "SELECT slug, title, thumb_url, wide_thumb
             FROM videos
             WHERE status = 'published'
             ORDER BY views DESC, id DESC
             LIMIT ?",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        let needle = format!("%{}%", normalize_needle(trimmed));
        sqlx::query_as::<_, VideoHelpRow>(
            "SELECT slug, title, thumb_url, wide_thumb
             FROM videos
             WHERE status = 'published' AND title LIKE ?
             ORDER BY views DESC, id DESC
             LIMIT ?",
        )
        .bind(&needle)
        .bind(limit)
        .fetch_all(pool)
        .await?
    };

    Ok(SearchHelpResponse {
        pornstars: pornstars
            .into_iter()
            .map(|r| SearchHelpPornstarItem {
                url_pornstar: pornstar_profile_path(&r.slug),
                name: String::new(),
                orig_name: r.display_name,
                thumb: pornstar_thumb(&r.thumb_path, &r.slug),
                count_videos: r.video_count.to_string(),
            })
            .collect(),
        channels: channels
            .into_iter()
            .map(|r| SearchHelpChannelItem {
                url: channel_profile_path(&r.slug),
                orig_name: r.title,
                rus_name: String::new(),
                thumb: channel_thumb(&r.thumb_path, &r.slug),
                count_videos: r.video_count.to_string(),
            })
            .collect(),
        videos: videos
            .into_iter()
            .map(|r| SearchHelpVideoItem {
                url: video_page_path(&r.slug),
                title: r.title,
                thumb: r.thumb_url,
                widethumb: if r.wide_thumb != 0 { "1" } else { "0" }.to_string(),
            })
            .collect(),
        search_text: text.to_string(),
    })
}

fn pornstar_matches(p: &SeedPornstar, needle: &str) -> bool {
    p.display_name.to_lowercase().contains(needle) || p.slug.to_lowercase().contains(needle)
}

fn channel_matches(c: &SeedChannel, needle: &str) -> bool {
    c.title.to_lowercase().contains(needle) || c.slug.to_lowercase().contains(needle)
}

fn video_matches(v: &SeedVideo, needle: &str) -> bool {
    v.title.to_lowercase().contains(needle) || v.slug.to_lowercase().contains(needle)
}

/// Build the response from bundled fixtures. Empty queries surface the most
/// popular seed entries so the endpoint never returns an empty body.
pub fn search_help_from_seed(seed: &CatalogSeed, text: &str, limit: u32) -> SearchHelpResponse {
    let trimmed = text.trim();
    let limit = limit.clamp(1, 50) as usize;
    let needle = trimmed.to_lowercase();
    let match_all = trimmed.is_empty();

    let mut pornstars: Vec<&SeedPornstar> = seed
        .pornstars
        .iter()
        .filter(|p| match_all || pornstar_matches(p, &needle))
        .collect();
    pornstars.sort_by(|a, b| {
        b.week_views
            .cmp(&a.week_views)
            .then(b.video_count.cmp(&a.video_count))
            .then(a.display_name.cmp(&b.display_name))
    });
    pornstars.truncate(limit);

    let mut channels: Vec<&SeedChannel> = seed
        .channels
        .iter()
        .filter(|c| match_all || channel_matches(c, &needle))
        .collect();
    channels.sort_by(|a, b| {
        b.week_views
            .cmp(&a.week_views)
            .then(b.video_count.cmp(&a.video_count))
            .then(a.title.cmp(&b.title))
    });
    channels.truncate(limit);

    let mut videos: Vec<&SeedVideo> = seed
        .videos
        .iter()
        .filter(|v| match_all || video_matches(v, &needle))
        .collect();
    videos.sort_by(|a, b| b.views.cmp(&a.views).then(b.id.cmp(&a.id)));
    videos.truncate(limit);

    SearchHelpResponse {
        pornstars: pornstars
            .into_iter()
            .map(|p| SearchHelpPornstarItem {
                url_pornstar: pornstar_profile_path(&p.slug),
                name: String::new(),
                orig_name: p.display_name.clone(),
                thumb: pornstar_thumb(&p.thumb_path, &p.slug),
                count_videos: p.video_count.to_string(),
            })
            .collect(),
        channels: channels
            .into_iter()
            .map(|c| SearchHelpChannelItem {
                url: channel_profile_path(&c.slug),
                orig_name: c.title.clone(),
                rus_name: String::new(),
                thumb: channel_thumb(&c.thumb_path, &c.slug),
                count_videos: c.video_count.to_string(),
            })
            .collect(),
        videos: videos
            .into_iter()
            .map(|v| SearchHelpVideoItem {
                url: video_page_path(&v.slug),
                title: v.title.clone(),
                thumb: v.thumb_url.clone(),
                widethumb: if v.wide_thumb != 0 { "1" } else { "0" }.to_string(),
            })
            .collect(),
        search_text: text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::load_catalog_seed;

    #[test]
    fn seed_search_matches_production_keys_and_url_shapes() {
        let seed = load_catalog_seed().expect("seed json");
        let resp = search_help_from_seed(&seed, "milf", SEARCH_HELP_GROUP_LIMIT);
        assert_eq!(resp.search_text, "milf");

        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("pornstars").is_some());
        assert!(json.get("channels").is_some());
        assert!(json.get("videos").is_some());
        assert!(json.get("search_text").is_some());

        if let Some(p) = resp.pornstars.first() {
            assert!(p.url_pornstar.starts_with("/pornstar/"));
            assert!(p.thumb.contains("fox-images/pornstars"));
            assert!(p.count_videos.chars().all(|c| c.is_ascii_digit()));
        }
        if let Some(c) = resp.channels.first() {
            assert!(c.url.starts_with("/channel/"));
            assert!(c.thumb.contains("fox-images/channels"));
        }
        if let Some(v) = resp.videos.first() {
            assert!(v.url.starts_with("/video/"));
            assert!(v.url.ends_with(".html"));
            assert!(v.widethumb == "1" || v.widethumb == "0");
        }
    }

    #[test]
    fn empty_query_returns_default_suggestions() {
        let seed = load_catalog_seed().expect("seed json");
        let resp = search_help_from_seed(&seed, "", SEARCH_HELP_GROUP_LIMIT);
        assert_eq!(resp.search_text, "");
        assert!(!resp.is_empty(), "empty query should surface defaults");
        assert!(!resp.pornstars.is_empty());
        assert!(!resp.channels.is_empty());
        assert!(!resp.videos.is_empty());
    }

    #[test]
    fn search_text_echo_preserves_raw_input() {
        let seed = load_catalog_seed().expect("seed json");
        let resp = search_help_from_seed(&seed, "  MiLf  ", SEARCH_HELP_GROUP_LIMIT);
        assert_eq!(resp.search_text, "  MiLf  ");
    }

    #[test]
    fn limit_caps_group_sizes() {
        let seed = load_catalog_seed().expect("seed json");
        let resp = search_help_from_seed(&seed, "", 2);
        assert!(resp.pornstars.len() <= 2);
        assert!(resp.channels.len() <= 2);
        assert!(resp.videos.len() <= 2);
    }
}
