//! Category/tag slug listing view and tags hub view models.

use crate::models::pagination::{
    build_page_nav, listing_path_with_query, HdFilter, ListingKind, PageNavItem, PaginationMeta,
    SortKey,
};
use crate::models::taxonomy::{ListingSort, TagRow, TaxonomyListingHeader};
use crate::models::video::VideoThumb;

const LAZY: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

/// Rendered fragments for a `/{slug}` (category or tag) video listing page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlugListingView {
    pub grid_html: String,
    pub filter_all_html: String,
    pub filter_hd_html: String,
    pub sort_links_html: String,
    pub page_nav_html: String,
    pub seo_blurb_html: String,
}

impl SlugListingView {
    pub fn build(
        header: &TaxonomyListingHeader,
        videos: Vec<VideoThumb>,
        meta: &PaginationMeta,
        sort: &SortKey,
        hd: HdFilter,
        listing: ListingKind,
    ) -> Self {
        let current_sort = match sort {
            SortKey::Video(s) => *s,
            _ => ListingSort::Latest,
        };
        let normalized_sort = SortKey::Video(current_sort);
        let all_href =
            listing_path_with_query(&listing, meta.page, &normalized_sort, HdFilter::All);
        let hd_href =
            listing_path_with_query(&listing, meta.page, &normalized_sort, HdFilter::HdOnly);
        let all_class = if hd == HdFilter::All { " active " } else { "" };
        let hd_class = if hd == HdFilter::HdOnly {
            " active "
        } else {
            ""
        };
        let seo_blurb_html = header
            .description
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(|d| {
                format!(
                    r#"<div class="toptext-container"><p class="toptext">{}</p></div>"#,
                    html_escape(d)
                )
            })
            .unwrap_or_default();
        let page_nav = build_page_nav(&listing, meta.page, meta.total_pages, &normalized_sort, hd);
        Self {
            grid_html: render_grid(&videos),
            filter_all_html: format!(r#"<a class="{all_class}" href="{all_href}">All</a>"#),
            filter_hd_html: format!(
                r#"<a class="{hd_class}" href="{hd_href}" rel="nofollow">HD</a>"#
            ),
            sort_links_html: render_sort_links(meta.page, current_sort, hd, &listing),
            page_nav_html: render_page_nav(&page_nav),
            seo_blurb_html,
        }
    }
}

fn render_grid(videos: &[VideoThumb]) -> String {
    videos
        .iter()
        .map(render_thumb)
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_thumb(v: &VideoThumb) -> String {
    let title = html_escape(&v.title);
    format!(
        concat!(
            r#"<div class="{cls}" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
            r#"<a class="thumb-in" href="{url}" target="_blank" itemprop="url"> "#,
            r#"<div class="thumb-img"> <div class="video-preview"></div> "#,
            r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" data-video="{preview}" alt="{title}" /> "#,
            r#"<div class="thumb-meta-top fx-row"><span class="ttime">{duration}</span></div> "#,
            r#"<div class="thumb-meta-bottom fx-row">"#,
            r#"<span class="tview">{views}</span>"#,
            r#"<span class="tlike"><i class="fa fa-star"></i>{likes}</span>"#,
            r#"</div></div> <div class="thumb-title" itemprop="name">{title}</div> </a> </div>"#
        ),
        cls = v.thumb_css_class(),
        url = v.page_path(),
        lazy = LAZY,
        thumb = v.thumb_url,
        preview = v.preview_mp4,
        title = title,
        duration = v.duration_label(),
        views = v.views_label(),
        likes = v.likes_label(),
    )
}

fn render_sort_links(
    page: u32,
    current: ListingSort,
    hd: HdFilter,
    listing: &ListingKind,
) -> String {
    let mut out = String::from(r#"<div class="select_sort">"#);
    for (label, key) in [
        ("Newest", ListingSort::Latest),
        ("Most Viewed", ListingSort::MostViewed),
        ("Most Commented", ListingSort::MostCommented),
    ] {
        let href = listing_path_with_query(listing, page, &SortKey::Video(key), hd);
        let selected = if current == key { " selected " } else { "" };
        let rel = if current == key {
            ""
        } else {
            r#" rel="nofollow""#
        };
        out.push_str(&format!(
            r#" <a class="{selected}" href="{href}"{rel}><i class="fa fa-check"></i>{label}</a>"#
        ));
    }
    out.push_str("</div>");
    out
}

fn render_page_nav(items: &[PageNavItem]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let mut out = String::from(r#"<div class="page_nav"><ul class="pagination">"#);
    for item in items {
        match item {
            PageNavItem::Current(p) => {
                out.push_str(&format!(r#"<li class="active"><span>{p}</span></li>"#))
            }
            PageNavItem::Link { page, href } => out.push_str(&format!(
                r#"<li class="pag-num"><a href="{href}">{page}</a></li>"#
            )),
            PageNavItem::Ellipsis => out.push_str(r#"<li class="dots">...</li>"#),
            PageNavItem::Previous { href, .. } => out.push_str(&format!(
                r#"<li class="previous"><a href="{href}" rel="prev">‹ Prev</a></li>"#
            )),
            PageNavItem::Next { href, rel } => out.push_str(&format!(
                r#"<li class="next"><a href="{href}" rel="{rel}">Next ›</a></li>"#
            )),
        }
    }
    out.push_str("</ul></div>");
    out
}

/// A single tag card on the `/tags` hub grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagsHubCard {
    pub slug: String,
    pub display_name: String,
    pub listing_url: String,
    pub video_count: u32,
}

/// Rendered fragments for the `/tags` hub page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TagsHubView {
    pub cards: Vec<TagsHubCard>,
    pub list_html: String,
}

impl TagsHubView {
    pub fn build(tags: &[TagRow]) -> Self {
        let cards: Vec<TagsHubCard> = tags
            .iter()
            .map(|t| TagsHubCard {
                slug: t.slug.clone(),
                display_name: t.display_name.clone(),
                listing_url: format!("/{}", t.slug),
                video_count: t.video_count,
            })
            .collect();
        let list_html = render_tags_list(&cards);
        Self { cards, list_html }
    }
}

fn render_tags_list(cards: &[TagsHubCard]) -> String {
    if cards.is_empty() {
        return String::new();
    }
    let mut out = String::from(r#"<ul class="tags-list-all">"#);
    for c in cards {
        out.push_str(&format!(
            r#"<li class="tag-item"><a href="{url}"><i class="fa fa-tag"></i>{name}</a><span class="count-tags">{count}</span></li>"#,
            url = c.listing_url,
            name = html_escape(&c.display_name),
            count = c.video_count,
        ));
    }
    out.push_str("</ul>");
    out
}

fn html_escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pagination::{page_request, EntityIndexKind, ListingQueryParams};
    use crate::models::taxonomy::{CategoryRow, ListingSlugKind};

    fn milf_header() -> TaxonomyListingHeader {
        CategoryRow {
            id: 1,
            slug: "milf".into(),
            display_name: "MILF".into(),
            description: Some("Hot MILF scenes.".into()),
            thumb_url: None,
            video_count: 12,
            intro_html: None,
            sort_order: 0,
            is_active: true,
        }
        .to_listing_header()
    }

    fn thumb(id: u64) -> VideoThumb {
        VideoThumb {
            id,
            slug: format!("scene-{id}"),
            title: format!("Scene {id}"),
            duration_seconds: 600,
            thumb_url: format!("https://c.foxporn.tv/fox-images/videos/scene-{id}.jpg"),
            preview_mp4: format!("https://c.foxporn.tv/fox-images/videos/m-scene-{id}.mp4"),
            views: 1000,
            likes_percent: 90,
            comments: 3,
            published_at: None,
            is_hd: true,
            wide_thumb: true,
        }
    }

    #[test]
    fn slug_listing_grid_links_to_video_pages() {
        let header = milf_header();
        let listing = ListingKind::CategorySlug {
            slug: "milf".into(),
        };
        let q = ListingQueryParams::default();
        let (spec, meta) = page_request(listing.clone(), None, &q, 12, None).unwrap();
        let view = SlugListingView::build(
            &header,
            vec![thumb(1), thumb(2)],
            &meta,
            &spec.sort,
            HdFilter::All,
            listing,
        );
        assert!(view.grid_html.contains("/video/scene-1.html"));
        assert!(view.grid_html.contains("thumb vid"));
        assert!(view.seo_blurb_html.contains("Hot MILF scenes."));
        assert!(view.filter_all_html.contains(" active "));
        assert_eq!(header.kind, ListingSlugKind::Category);
    }

    #[test]
    fn slug_listing_page_nav_uses_slug_prefix() {
        let listing = ListingKind::CategorySlug {
            slug: "milf".into(),
        };
        let q = ListingQueryParams::default();
        let (spec, meta) = page_request(listing.clone(), Some("1"), &q, 54 * 3, None).unwrap();
        let view = SlugListingView::build(
            &milf_header(),
            vec![],
            &meta,
            &spec.sort,
            HdFilter::All,
            listing,
        );
        assert!(view.page_nav_html.contains("/milf/2"));
        assert!(!view.page_nav_html.contains("\"/2\""));
    }

    #[test]
    fn tags_hub_builds_cards_and_list() {
        let tags = vec![TagRow {
            id: 1,
            slug: "hot-mom".into(),
            display_name: "Hot Mom".into(),
            description: None,
            thumb_url: None,
            video_count: 7,
            weekly_views: 100,
            is_active: true,
        }];
        let view = TagsHubView::build(&tags);
        assert_eq!(view.cards.len(), 1);
        assert!(view.list_html.contains("/hot-mom"));
        assert!(view.list_html.contains("Hot Mom"));
        // sanity: entity index kind import exercised
        let _ = EntityIndexKind::Tags;
    }
}
