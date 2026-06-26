//! Pornstar/channel profile listing view (shared header + video grid).

use crate::models::entities::EntityProfileHeader;
use crate::models::pagination::{
    build_page_nav, listing_path_with_query, HdFilter, ListingKind, PageNavItem, PaginationMeta,
    SortKey,
};
use crate::models::taxonomy::ListingSort;
use crate::models::video::VideoThumb;

const LAZY: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

const VERIFIED_BADGE_SVG: &str = r##"<svg fill="none" viewBox="0 0 12.444 12.444"><circle cx="6.222" cy="6.222" r="6.222" fill="#0F8114"></circle><path d="M5.185 7.03 4.141 5.898l-.892.778L5.185 8.49 9.333 4.6l-.691-.647z" fill="#fff"></path></svg>"##;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityProfileView {
    pub header_html: String,
    pub grid_html: String,
    pub filter_all_html: String,
    pub filter_hd_html: String,
    pub sort_links_html: String,
    pub page_nav_html: String,
    pub seo_blurb_html: String,
}

impl EntityProfileView {
    pub fn build(
        header: &EntityProfileHeader,
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
        let page_nav = build_page_nav(&listing, meta.page, meta.total_pages, &normalized_sort, hd);
        Self {
            header_html: render_profile_header(header),
            grid_html: render_grid(&videos),
            filter_all_html: format!(r#"<a class="{all_class}" href="{all_href}">All</a>"#),
            filter_hd_html: format!(
                r#"<a class="{hd_class}" href="{hd_href}" rel="nofollow">HD</a>"#
            ),
            sort_links_html: render_sort_links(meta.page, current_sort, hd, &listing),
            page_nav_html: render_page_nav(&page_nav),
            seo_blurb_html: String::new(),
        }
    }
}

pub fn render_profile_header(header: &EntityProfileHeader) -> String {
    let banner = header.banner_url.as_deref().unwrap_or("");
    let avatar = header.avatar_url.as_deref().unwrap_or("");
    if banner.is_empty() && avatar.is_empty() {
        return String::new();
    }
    let name = html_escape(&header.display_name);
    let badge = if header.show_verified_badge {
        VERIFIED_BADGE_SVG
    } else {
        ""
    };
    let avatar_alt = format!("{name} {}", header.avatar_alt_suffix);
    format!(
        concat!(
            r#"<div id="head-banner" style="padding-bottom: 20.4082%;">"#,
            r#"<img src="{banner}" alt="{name}">"#,
            r#"<div id="head-avatar"><img src="{avatar}" alt="{avatar_alt}"></div>"#,
            r#"<div id="head-gradient"><div id="head-name">{name}{badge}</div></div>"#,
            r#"</div>"#
        ),
        banner = html_escape(banner),
        avatar = html_escape(avatar),
        name = name,
        avatar_alt = html_escape(&avatar_alt),
        badge = badge,
    )
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
