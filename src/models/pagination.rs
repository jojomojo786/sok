//! Centralized pagination, sort, and HD filter helpers for listing handlers.

use crate::models::taxonomy::{ListingSort, SearchSort};
use serde::Deserialize;
use std::collections::HashMap;

pub const DEFAULT_HOME_VIDEO_PER_PAGE: u32 = 54;
pub const DEFAULT_ENTITY_INDEX_PER_PAGE: u32 = 60;
pub const PAGINATION_WINDOW: u32 = 9;
pub const DEFAULT_SITE_BASE_URL: &str = "https://pornsok.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListingKind {
    Home,
    CategoriesIndex,
    EntityIndex(EntityIndexKind),
    CategorySlug { slug: String },
    EntityProfile(EntityProfileKind, String),
    Search { query_slug: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityIndexKind {
    Pornstars,
    Channels,
    Tags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityProfileKind {
    Pornstar,
    Channel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntitySortKey {
    #[default]
    Trending,
    VideoCount,
    Alphabetical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortKey {
    Video(ListingSort),
    Search(SearchSort),
    Entity(EntitySortKey),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HdFilter {
    #[default]
    All,
    HdOnly,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListingQueryParams {
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub hd: Option<String>,
    #[serde(default)]
    pub page: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSpec {
    pub listing: ListingKind,
    pub page: u32,
    pub per_page: u32,
    pub sort: SortKey,
    pub hd: HdFilter,
    pub offset: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub offset: u32,
    pub limit: u32,
    pub has_previous: bool,
    pub has_next: bool,
    pub rel_prev: Option<String>,
    pub rel_next: Option<String>,
    pub canonical_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaginationError {
    InvalidPage(String),
    PageOutOfRange { page: u32, total_pages: u32 },
}

impl std::fmt::Display for PaginationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaginationError::InvalidPage(msg) => write!(f, "invalid page: {msg}"),
            PaginationError::PageOutOfRange { page, total_pages } => {
                write!(f, "page {page} out of range (total pages: {total_pages})")
            }
        }
    }
}

impl std::error::Error for PaginationError {}

impl ListingKind {
    pub fn default_per_page(&self) -> u32 {
        match self {
            ListingKind::Home
            | ListingKind::CategorySlug { .. }
            | ListingKind::EntityProfile(..)
            | ListingKind::Search { .. } => DEFAULT_HOME_VIDEO_PER_PAGE,
            ListingKind::CategoriesIndex | ListingKind::EntityIndex(..) => {
                DEFAULT_ENTITY_INDEX_PER_PAGE
            }
        }
    }

    fn base_path(&self) -> String {
        match self {
            ListingKind::Home => "/".to_string(),
            ListingKind::CategoriesIndex => "/categories".to_string(),
            ListingKind::EntityIndex(EntityIndexKind::Pornstars) => "/pornstars".to_string(),
            ListingKind::EntityIndex(EntityIndexKind::Channels) => "/channels".to_string(),
            ListingKind::EntityIndex(EntityIndexKind::Tags) => "/tags".to_string(),
            ListingKind::CategorySlug { slug } => format!("/{slug}"),
            ListingKind::EntityProfile(EntityProfileKind::Pornstar, slug) => {
                format!("/pornstar/{slug}")
            }
            ListingKind::EntityProfile(EntityProfileKind::Channel, slug) => {
                format!("/channel/{slug}")
            }
            ListingKind::Search { query_slug } => format!("/videos/{query_slug}"),
        }
    }
}

impl ListingQueryParams {
    pub fn from_query_map(map: &HashMap<String, String>) -> Self {
        Self {
            sort: map.get("sort").cloned(),
            hd: map.get("hd").cloned(),
            page: map.get("page").cloned(),
        }
    }

    pub fn parse_sort_for_listing(&self, listing: &ListingKind) -> SortKey {
        let raw = self.sort.as_deref();
        match listing {
            ListingKind::EntityIndex(_) | ListingKind::CategoriesIndex => {
                SortKey::Entity(parse_entity_sort(raw.unwrap_or("")))
            }
            ListingKind::Search { .. } => SortKey::Search(SearchSort::from_query(raw)),
            ListingKind::Home
            | ListingKind::CategorySlug { .. }
            | ListingKind::EntityProfile(..) => SortKey::Video(ListingSort::from_query(raw)),
        }
    }

    pub fn parse_hd(&self) -> HdFilter {
        parse_hd_filter(self.hd.as_deref())
    }

    pub fn parse_page_number(&self) -> Result<Option<u32>, PaginationError> {
        match self.page.as_deref() {
            None | Some("") => Ok(None),
            Some(raw) => parse_page_number(raw).map(Some),
        }
    }
}

impl SortKey {
    pub fn default_for_listing(listing: &ListingKind) -> Self {
        match listing {
            ListingKind::EntityIndex(_) | ListingKind::CategoriesIndex => {
                SortKey::Entity(EntitySortKey::default())
            }
            ListingKind::Search { .. } => SortKey::Search(SearchSort::default()),
            _ => SortKey::Video(ListingSort::Latest),
        }
    }

    pub fn query_pairs(&self, hd: HdFilter) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        match self {
            SortKey::Video(ListingSort::MostViewed) => pairs.push(("sort", "mv".into())),
            SortKey::Video(ListingSort::MostCommented) => pairs.push(("sort", "mc".into())),
            SortKey::Video(ListingSort::Latest) => {}
            SortKey::Search(SearchSort::Newest) => pairs.push(("sort", "recent".into())),
            SortKey::Search(SearchSort::MostViewed) => pairs.push(("sort", "mv".into())),
            SortKey::Search(SearchSort::MostCommented) => pairs.push(("sort", "mc".into())),
            SortKey::Search(SearchSort::Relevant) => {}
            SortKey::Entity(EntitySortKey::VideoCount) => {
                pairs.push(("sort", "videocount".into()));
            }
            SortKey::Entity(EntitySortKey::Alphabetical) => {
                pairs.push(("sort", "alphabetical".into()));
            }
            SortKey::Entity(EntitySortKey::Trending) => {}
            SortKey::None => {}
        }
        if hd == HdFilter::HdOnly {
            pairs.push(("hd", "1".into()));
        }
        pairs
    }
}

pub fn parse_page_number(raw: &str) -> Result<u32, PaginationError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PaginationError::InvalidPage("empty page".into()));
    }
    if !trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err(PaginationError::InvalidPage(format!(
            "non-numeric page: {trimmed}"
        )));
    }
    let page: u32 = trimmed
        .parse()
        .map_err(|_| PaginationError::InvalidPage(trimmed.to_string()))?;
    if page == 0 {
        return Err(PaginationError::InvalidPage("page must be >= 1".into()));
    }
    Ok(page)
}

pub fn parse_entity_sort(raw: &str) -> EntitySortKey {
    match raw.trim().to_ascii_lowercase().as_str() {
        "videocount" | "video_count" | "videos" | "count" => EntitySortKey::VideoCount,
        "alphabetical" | "alpha" | "name" => EntitySortKey::Alphabetical,
        _ => EntitySortKey::Trending,
    }
}

pub fn parse_hd_filter(raw: Option<&str>) -> HdFilter {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => HdFilter::All,
        Some("1") | Some("true") | Some("yes") | Some("on") => HdFilter::HdOnly,
        _ => HdFilter::All,
    }
}

pub fn total_pages(total_items: u64, per_page: u32) -> u32 {
    if per_page == 0 {
        return 0;
    }
    if total_items == 0 {
        return 1;
    }
    ((total_items + u64::from(per_page) - 1) / u64::from(per_page)) as u32
}

pub fn build_page_spec(
    listing: ListingKind,
    page: u32,
    total_items: u64,
    query: &ListingQueryParams,
    per_page: Option<u32>,
) -> Result<PageSpec, PaginationError> {
    if page == 0 {
        return Err(PaginationError::InvalidPage("page must be >= 1".into()));
    }
    let per_page = per_page.unwrap_or_else(|| listing.default_per_page());
    let total_pages = total_pages(total_items, per_page);
    if page > total_pages {
        return Err(PaginationError::PageOutOfRange { page, total_pages });
    }
    let offset = (page - 1).saturating_mul(per_page);
    Ok(PageSpec {
        listing: listing.clone(),
        page,
        per_page,
        sort: query.parse_sort_for_listing(&listing),
        hd: query.parse_hd(),
        offset,
        limit: per_page,
    })
}

pub fn resolve_page(
    path_page: Option<&str>,
    query: &ListingQueryParams,
) -> Result<u32, PaginationError> {
    if let Some(seg) = path_page {
        return parse_page_number(seg);
    }
    if let Some(p) = query.parse_page_number()? {
        return Ok(p);
    }
    Ok(1)
}

pub fn pagination_meta(spec: &PageSpec, total_items: u64, site_base: &str) -> PaginationMeta {
    let total_pages = total_pages(total_items, spec.per_page);
    let has_previous = spec.page > 1;
    let has_next = spec.page < total_pages;
    // Live PornsOK canonicals use the unfiltered base listing URL; rel prev/next keep filters.
    let canonical_path = listing_path_with_query(
        &spec.listing,
        spec.page,
        &SortKey::default_for_listing(&spec.listing),
        HdFilter::All,
    );
    let base = normalize_site_base(site_base);
    let rel_prev = has_previous.then(|| {
        absolute_url(
            &base,
            &listing_path_with_query(&spec.listing, spec.page - 1, &spec.sort, spec.hd),
        )
    });
    let rel_next = has_next.then(|| {
        absolute_url(
            &base,
            &listing_path_with_query(&spec.listing, spec.page + 1, &spec.sort, spec.hd),
        )
    });
    PaginationMeta {
        page: spec.page,
        per_page: spec.per_page,
        total_items,
        total_pages,
        offset: spec.offset,
        limit: spec.limit,
        has_previous,
        has_next,
        rel_prev,
        rel_next,
        canonical_path,
    }
}

pub fn page_request(
    listing: ListingKind,
    path_page: Option<&str>,
    query: &ListingQueryParams,
    total_items: u64,
    site_base: Option<&str>,
) -> Result<(PageSpec, PaginationMeta), PaginationError> {
    let page = resolve_page(path_page, query)?;
    let spec = build_page_spec(listing, page, total_items, query, None)?;
    let base = site_base.unwrap_or(DEFAULT_SITE_BASE_URL);
    let meta = pagination_meta(&spec, total_items, base);
    Ok((spec, meta))
}

pub fn listing_path_with_query(
    listing: &ListingKind,
    page: u32,
    sort: &SortKey,
    hd: HdFilter,
) -> String {
    let path = listing_page_path(listing, page);
    let query = build_query_string(sort, hd);
    if query.is_empty() {
        path
    } else {
        format!("{path}?{query}")
    }
}

pub fn listing_page_path(listing: &ListingKind, page: u32) -> String {
    match listing {
        ListingKind::Home => {
            if page <= 1 {
                "/".to_string()
            } else {
                format!("/{page}")
            }
        }
        _ => {
            let base = listing.base_path();
            if page <= 1 {
                base
            } else {
                format!("{base}/{page}")
            }
        }
    }
}

pub fn build_query_string(sort: &SortKey, hd: HdFilter) -> String {
    let pairs = sort.query_pairs(hd);
    if pairs.is_empty() {
        return String::new();
    }
    pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn absolute_url(site_base: &str, path_and_query: &str) -> String {
    let base = normalize_site_base(site_base);
    if path_and_query == "/" {
        return format!("{base}/");
    }
    let path = path_and_query.strip_prefix('/').unwrap_or(path_and_query);
    format!("{base}/{path}")
}

fn normalize_site_base(site_base: &str) -> String {
    site_base.trim_end_matches('/').to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNavItem {
    Current(u32),
    Link { page: u32, href: String },
    Ellipsis,
    Previous { href: String, rel: &'static str },
    Next { href: String, rel: &'static str },
}

pub fn build_page_nav(
    listing: &ListingKind,
    current: u32,
    total_pages: u32,
    sort: &SortKey,
    hd: HdFilter,
) -> Vec<PageNavItem> {
    if total_pages == 0 {
        return Vec::new();
    }
    let mut items = Vec::new();
    if current > 1 {
        items.push(PageNavItem::Previous {
            href: listing_path_with_query(listing, current - 1, sort, hd),
            rel: "prev",
        });
    }
    let window_end = PAGINATION_WINDOW.min(total_pages);
    for p in 1..=window_end {
        push_page_link(&mut items, listing, current, p, sort, hd);
    }
    if total_pages > PAGINATION_WINDOW + 1 {
        items.push(PageNavItem::Ellipsis);
        push_page_link(&mut items, listing, current, total_pages, sort, hd);
    } else if total_pages > window_end {
        for p in (window_end + 1)..=total_pages {
            push_page_link(&mut items, listing, current, p, sort, hd);
        }
    }
    if current < total_pages {
        items.push(PageNavItem::Next {
            href: listing_path_with_query(listing, current + 1, sort, hd),
            rel: "next",
        });
    }
    items
}

fn push_page_link(
    items: &mut Vec<PageNavItem>,
    listing: &ListingKind,
    current: u32,
    page: u32,
    sort: &SortKey,
    hd: HdFilter,
) {
    if page == current {
        items.push(PageNavItem::Current(page));
    } else {
        items.push(PageNavItem::Link {
            page,
            href: listing_path_with_query(listing, page, sort, hd),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_numeric_paths() {
        let listing = ListingKind::Home;
        assert_eq!(listing_page_path(&listing, 1), "/");
        assert_eq!(listing_page_path(&listing, 2), "/2");
    }

    #[test]
    fn pornstars_prefix_paths() {
        let listing = ListingKind::EntityIndex(EntityIndexKind::Pornstars);
        assert_eq!(listing_page_path(&listing, 1), "/pornstars");
        assert_eq!(listing_page_path(&listing, 2), "/pornstars/2");
    }

    #[test]
    fn slug_and_search_paths() {
        let cat = ListingKind::CategorySlug {
            slug: "milf".into(),
        };
        assert_eq!(listing_page_path(&cat, 5), "/milf/5");
        let search = ListingKind::Search {
            query_slug: "test".into(),
        };
        assert_eq!(listing_page_path(&search, 1), "/videos/test");
    }

    #[test]
    fn offset_and_defaults() {
        let q = ListingQueryParams::default();
        let spec = build_page_spec(ListingKind::Home, 3, 200, &q, None).unwrap();
        assert_eq!(spec.offset, 108);
        assert_eq!(spec.limit, 54);
    }

    #[test]
    fn sort_hd_urls() {
        let q = ListingQueryParams {
            sort: Some("mv".into()),
            hd: Some("1".into()),
            page: None,
        };
        let sort = q.parse_sort_for_listing(&ListingKind::Home);
        let path = listing_path_with_query(&ListingKind::Home, 2, &sort, q.parse_hd());
        assert_eq!(path, "/2?sort=mv&hd=1");
    }

    #[test]
    fn canonical_strips_sort_and_hd_query_params() {
        let q = ListingQueryParams {
            sort: Some("mv".into()),
            hd: Some("1".into()),
            page: None,
        };
        let listing = ListingKind::CategorySlug {
            slug: "milf".into(),
        };
        let spec = build_page_spec(listing.clone(), 1, 120, &q, None).unwrap();
        let meta = pagination_meta(&spec, 120, DEFAULT_SITE_BASE_URL);
        assert_eq!(meta.canonical_path, "/milf");

        let q_search = ListingQueryParams {
            sort: Some("recent".into()),
            hd: Some("1".into()),
            page: None,
        };
        let search = ListingKind::Search {
            query_slug: "test".into(),
        };
        let spec_search = build_page_spec(search.clone(), 2, 120, &q_search, None).unwrap();
        let meta_search = pagination_meta(&spec_search, 120, DEFAULT_SITE_BASE_URL);
        assert_eq!(meta_search.canonical_path, "/videos/test/2");
    }

    #[test]
    fn rel_links_preserve_sort_and_hd_filters() {
        let q = ListingQueryParams {
            sort: Some("mv".into()),
            hd: Some("1".into()),
            page: None,
        };
        let listing = ListingKind::Home;
        let spec = build_page_spec(listing, 2, 540, &q, None).unwrap();
        let meta = pagination_meta(&spec, 540, DEFAULT_SITE_BASE_URL);
        assert_eq!(
            meta.rel_prev.as_deref(),
            Some("https://pornsok.com/?sort=mv&hd=1")
        );
        assert_eq!(
            meta.rel_next.as_deref(),
            Some("https://pornsok.com/3?sort=mv&hd=1")
        );
    }

    #[test]
    fn rel_links() {
        let q = ListingQueryParams::default();
        let (_, meta) = page_request(ListingKind::Home, Some("1"), &q, 540, None).unwrap();
        assert_eq!(meta.rel_next.as_deref(), Some("https://pornsok.com/2"));
        let (_, meta2) = page_request(ListingKind::Home, Some("2"), &q, 540, None).unwrap();
        assert_eq!(meta2.rel_prev.as_deref(), Some("https://pornsok.com/"));
    }

    #[test]
    fn page_nav_shape() {
        let nav = build_page_nav(
            &ListingKind::Home,
            1,
            1239,
            &SortKey::Video(ListingSort::Latest),
            HdFilter::All,
        );
        assert!(nav.iter().any(|i| matches!(i, PageNavItem::Ellipsis)));
        assert!(matches!(nav.last(), Some(PageNavItem::Next { .. })));
    }
}
