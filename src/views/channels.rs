//! Channels index view models.

use crate::models::entities::EntityIndexCard;
use crate::models::pagination::{
    build_page_nav, listing_path_with_query, EntityIndexKind, EntitySortKey, HdFilter, ListingKind,
    PageNavItem, PaginationMeta, SortKey,
};

const LAZY: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelCardView {
    pub slug: String,
    pub display_name: String,
    pub thumb_url: String,
    pub channel_url: String,
    pub video_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelSortLink {
    pub label: &'static str,
    pub href: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelsIndexView {
    pub cards: Vec<ChannelCardView>,
    pub sort_links: Vec<ChannelSortLink>,
    pub grid_html: String,
    pub sort_links_html: String,
    pub page_nav_html: String,
    pub search_type: &'static str,
}

impl Default for ChannelsIndexView {
    fn default() -> Self {
        Self {
            cards: Vec::new(),
            sort_links: Vec::new(),
            grid_html: String::new(),
            sort_links_html: String::new(),
            page_nav_html: String::new(),
            search_type: "channels",
        }
    }
}

impl ChannelsIndexView {
    pub fn build(
        entity_cards: Vec<EntityIndexCard>,
        meta: &PaginationMeta,
        sort: &SortKey,
        cdn_base: &str,
    ) -> Self {
        let cards: Vec<ChannelCardView> = entity_cards
            .iter()
            .map(|c| ChannelCardView {
                slug: c.slug.clone(),
                display_name: c.display_name.clone(),
                thumb_url: c.thumb_url(cdn_base),
                channel_url: format!("/channel/{}", c.slug),
                video_count: c.video_count,
            })
            .collect();
        let listing = ListingKind::EntityIndex(EntityIndexKind::Channels);
        let hd = HdFilter::All;
        let sort_links = entity_sort_links(meta.page, sort, hd);
        let page_nav = build_page_nav(&listing, meta.page, meta.total_pages, sort, hd);
        Self {
            cards: cards.clone(),
            sort_links: sort_links.clone(),
            grid_html: render_grid(&cards),
            sort_links_html: render_sort_links(&sort_links),
            page_nav_html: render_page_nav(&page_nav),
            search_type: "channels",
        }
    }
}

fn render_card(card: &ChannelCardView) -> String {
    let name = html_escape(&card.display_name);
    format!(
        concat!(
            r#"<div class="thumb cat" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
            r#"<a href="{url}" class="thumb-in" itemprop="url"> "#,
            r#"<div class="thumb-img"> "#,
            r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" alt="{name}" />"#,
            r#"<noscript><img class="thumb-cover" src="{thumb}" itemprop="contentUrl" alt="{name}" /></noscript> "#,
            "<span class=\"count-videos\"><svg fill=\"#fff\"><use xlink:href=\"#camera-svg\" /></svg>{count}</span> ",
            r#"</div> <div class="thumb-title" itemprop="name">{name}</div> </a> </div>"#
        ),
        url = card.channel_url,
        lazy = LAZY,
        thumb = card.thumb_url,
        name = name,
        count = card.video_count,
    )
}

fn render_grid(cards: &[ChannelCardView]) -> String {
    let mut out: String = cards.iter().map(render_card).collect::<Vec<_>>().join(" ");
    if cards.len() >= 8 {
        let preload: Vec<String> = cards
            .iter()
            .take(8)
            .map(|c| format!("\"{}\"", c.thumb_url))
            .collect();
        out.push_str(&format!(
            r#" <script type="text/javascript">var preload_thumbs=[{}];</script>"#,
            preload.join(",")
        ));
    }
    out
}

fn render_sort_links(links: &[ChannelSortLink]) -> String {
    let mut out = String::from(r#"<div class="select_sort">"#);
    for link in links {
        let selected = if link.selected { "selected" } else { "" };
        let rel = if link.selected {
            ""
        } else {
            r#" rel="nofollow""#
        };
        out.push_str(&format!(
            r#" <a href="{href}"{rel} class="{selected}"><i class="fa fa-check"></i>{label}</a>"#,
            href = link.href,
            rel = rel,
            selected = selected,
            label = link.label,
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
            PageNavItem::Current(page) => {
                out.push_str(&format!(r#"<li class="active"><span>{page}</span></li>"#));
            }
            PageNavItem::Link { page, href } => {
                out.push_str(&format!(
                    r#"<li class="pag-num"><a href="{href}">{page}</a></li>"#
                ));
            }
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

fn entity_sort_links(page: u32, sort: &SortKey, hd: HdFilter) -> Vec<ChannelSortLink> {
    let listing = ListingKind::EntityIndex(EntityIndexKind::Channels);
    let current = match sort {
        SortKey::Entity(key) => *key,
        _ => EntitySortKey::Trending,
    };
    [
        ("Most Popular", EntitySortKey::Trending),
        ("Video Count", EntitySortKey::VideoCount),
        ("Alphabetical", EntitySortKey::Alphabetical),
    ]
    .into_iter()
    .map(|(label, key)| ChannelSortLink {
        label,
        href: listing_path_with_query(&listing, page, &SortKey::Entity(key), hd),
        selected: key == current,
    })
    .collect()
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
    use crate::models::pagination::{page_request, ListingQueryParams};

    fn entity_card(slug: &str, name: &str) -> EntityIndexCard {
        EntityIndexCard {
            id: 1,
            slug: slug.into(),
            display_name: name.into(),
            thumb_path: format!("fox-images/channels/{slug}.jpg"),
            video_count: 42,
        }
    }

    #[test]
    fn card_html_preserves_dom_hooks() {
        let view = ChannelsIndexView::build(
            vec![entity_card("brazzers", "Brazzers")],
            &PaginationMeta {
                page: 1,
                per_page: 48,
                total_items: 1,
                total_pages: 1,
                offset: 0,
                limit: 48,
                has_previous: false,
                has_next: false,
                rel_prev: None,
                rel_next: None,
                canonical_path: "/channels".into(),
            },
            &SortKey::Entity(EntitySortKey::Trending),
            "https://c.foxporn.tv",
        );
        assert!(view.grid_html.contains("class=\"thumb cat\""));
        assert!(view.grid_html.contains("/channel/brazzers"));
        assert!(view.grid_html.contains("count-videos"));
        assert!(view.grid_html.contains("#camera-svg"));
        assert!(view.grid_html.contains("Brazzers"));
    }

    #[test]
    fn page_nav_uses_channels_prefix() {
        let q = ListingQueryParams::default();
        let (_, meta) = page_request(
            ListingKind::EntityIndex(EntityIndexKind::Channels),
            None,
            &q,
            83 * 48,
            None,
        )
        .unwrap();
        let view = ChannelsIndexView::build(
            vec![],
            &meta,
            &SortKey::Entity(EntitySortKey::Trending),
            "https://c.foxporn.tv",
        );
        assert!(view.page_nav_html.contains("page_nav"));
        assert!(view.page_nav_html.contains("/channels/2"));
    }
}
