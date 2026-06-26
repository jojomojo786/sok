//! JSON contract for in-page entity search (`/ajax/search_{type}`).

use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::fixtures::{load_catalog_seed, CatalogSeed};
use crate::models::entities::{
    channel_profile_path, media_url, pornstar_profile_path, search_channels_for_page,
    search_pornstars_for_page, PornstarPageSearchItem,
};

pub const ENTITY_PAGE_SEARCH_LIMIT: u32 = 120;

/// Whitelisted `search_type` values embedded in pornstars/channels index pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPageSearchType {
    Pornstars,
    Channels,
}

impl EntityPageSearchType {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "pstars" | "pornstars" => Some(Self::Pornstars),
            "channels" => Some(Self::Channels),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pornstars => "pstars",
            Self::Channels => "channels",
        }
    }

    pub fn handler_marker(self) -> &'static str {
        match self {
            Self::Pornstars => "search_pornstars",
            Self::Channels => "search_channels",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntityPageSearchResponse {
    pub search_text: String,
    pub items: Vec<PornstarPageSearchItem>,
}

pub async fn search_entities_for_page(
    pool: &sqlx::MySqlPool,
    search_type: EntityPageSearchType,
    text: &str,
    limit: u32,
    cdn_base: &str,
) -> Result<EntityPageSearchResponse, AppError> {
    let trimmed = text;
    if trimmed.trim().len() < 2 {
        return Ok(EntityPageSearchResponse {
            search_text: trimmed.to_string(),
            items: Vec::new(),
        });
    }

    let items = match search_type {
        EntityPageSearchType::Pornstars => search_pornstars_for_page(pool, trimmed, limit).await?,
        EntityPageSearchType::Channels => search_channels_for_page(pool, trimmed, limit).await?,
    };

    let items = items
        .into_iter()
        .map(|item| normalize_page_search_item(item, cdn_base))
        .collect();

    Ok(EntityPageSearchResponse {
        search_text: trimmed.to_string(),
        items,
    })
}

pub fn search_entities_for_page_from_seed(
    seed: &CatalogSeed,
    search_type: EntityPageSearchType,
    text: &str,
    limit: u32,
    cdn_base: &str,
) -> EntityPageSearchResponse {
    let trimmed = text.trim();
    if trimmed.len() < 2 {
        return EntityPageSearchResponse {
            search_text: text.to_string(),
            items: Vec::new(),
        };
    }

    let needle = trimmed.to_ascii_lowercase();
    let mut items = match search_type {
        EntityPageSearchType::Pornstars => seed
            .pornstars
            .iter()
            .filter(|p| {
                p.display_name.to_ascii_lowercase().contains(&needle)
                    || p.slug.to_ascii_lowercase().contains(&needle)
            })
            .map(|p| PornstarPageSearchItem {
                url: pornstar_profile_path(&p.slug),
                thumb: media_url(cdn_base, &p.thumb_path),
                orig_name: p.display_name.clone(),
                count_videos: p.video_count,
            })
            .collect::<Vec<_>>(),
        EntityPageSearchType::Channels => seed
            .channels
            .iter()
            .filter(|c| {
                c.title.to_ascii_lowercase().contains(&needle)
                    || c.slug.to_ascii_lowercase().contains(&needle)
            })
            .map(|c| PornstarPageSearchItem {
                url: channel_profile_path(&c.slug),
                thumb: media_url(cdn_base, &c.thumb_path),
                orig_name: c.title.clone(),
                count_videos: c.video_count,
            })
            .collect::<Vec<_>>(),
    };

    items.sort_by(|a, b| {
        b.count_videos
            .cmp(&a.count_videos)
            .then_with(|| a.orig_name.cmp(&b.orig_name))
    });
    items.truncate(limit as usize);

    EntityPageSearchResponse {
        search_text: text.to_string(),
        items,
    }
}

pub fn entity_page_search_fallback(
    search_type: EntityPageSearchType,
    text: &str,
    limit: u32,
    cdn_base: &str,
) -> Result<EntityPageSearchResponse, AppError> {
    let seed = load_catalog_seed()?;
    Ok(search_entities_for_page_from_seed(
        &seed,
        search_type,
        text,
        limit,
        cdn_base,
    ))
}

fn normalize_page_search_item(
    item: PornstarPageSearchItem,
    cdn_base: &str,
) -> PornstarPageSearchItem {
    PornstarPageSearchItem {
        thumb: absolute_thumb_url(&item.thumb, cdn_base),
        ..item
    }
}

fn absolute_thumb_url(thumb: &str, cdn_base: &str) -> String {
    let thumb = thumb.trim();
    if thumb.is_empty() {
        return String::new();
    }
    if thumb.starts_with("http://") || thumb.starts_with("https://") {
        thumb.to_string()
    } else {
        media_url(cdn_base, thumb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entities::DEFAULT_MEDIA_CDN;

    #[test]
    fn search_type_whitelist_matches_index_pages() {
        assert_eq!(
            EntityPageSearchType::parse("pstars"),
            Some(EntityPageSearchType::Pornstars)
        );
        assert_eq!(
            EntityPageSearchType::parse("channels"),
            Some(EntityPageSearchType::Channels)
        );
        assert_eq!(EntityPageSearchType::parse("videos"), None);
    }

    #[test]
    fn seed_search_returns_card_fields_for_pornstars() {
        let seed = load_catalog_seed().expect("seed");
        let resp = search_entities_for_page_from_seed(
            &seed,
            EntityPageSearchType::Pornstars,
            "ang",
            ENTITY_PAGE_SEARCH_LIMIT,
            DEFAULT_MEDIA_CDN,
        );
        assert_eq!(resp.search_text, "ang");
        assert!(!resp.items.is_empty());
        let item = &resp.items[0];
        assert!(item.url.starts_with("/pornstar/"));
        assert!(item.thumb.starts_with("https://"));
        assert!(!item.orig_name.is_empty());
    }

    #[test]
    fn short_query_returns_empty_items() {
        let seed = load_catalog_seed().expect("seed");
        let resp = search_entities_for_page_from_seed(
            &seed,
            EntityPageSearchType::Channels,
            "a",
            ENTITY_PAGE_SEARCH_LIMIT,
            DEFAULT_MEDIA_CDN,
        );
        assert_eq!(resp.search_text, "a");
        assert!(resp.items.is_empty());
    }
}
