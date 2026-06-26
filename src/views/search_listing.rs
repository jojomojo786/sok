//! `/videos/{query}` search results listing view.

use crate::models::pagination::{
    build_page_nav, listing_path_with_query, HdFilter, ListingKind, PageNavItem, PaginationMeta,
    SortKey,
};
use crate::models::taxonomy::SearchSort;
use crate::models::video::VideoThumb;

const LAZY: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchListingView {
    pub grid_html: String,
    pub filter_all_html: String,
    pub filter_hd_html: String,
    pub sort_links_html: String,
    pub sort_button_icon_html: String,
    pub page_nav_html: String,
    pub seo_blurb_html: String,
}

impl SearchListingView {
    pub fn build(
        videos: Vec<VideoThumb>,
        meta: &PaginationMeta,
        sort: &SortKey,
        hd: HdFilter,
        listing: ListingKind,
    ) -> Self {
        let current_sort = match sort {
            SortKey::Search(s) => *s,
            _ => SearchSort::Relevant,
        };
        let normalized_sort = SortKey::Search(current_sort);
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
            grid_html: render_grid(&videos),
            filter_all_html: format!(r#"<a class="{all_class}" href="{all_href}">All</a>"#),
            filter_hd_html: format!(
                r#"<a class="{hd_class}" href="{hd_href}" rel="nofollow">HD</a>"#
            ),
            sort_links_html: render_sort_links(meta.page, current_sort, hd, &listing),
            sort_button_icon_html: render_sort_button_icon(current_sort),
            page_nav_html: render_page_nav(&page_nav),
            seo_blurb_html: String::new(),
        }
    }
}

fn render_sort_button_icon(current: SearchSort) -> String {
    match current {
        SearchSort::Relevant => {
            r##"<i id="i-relevant"><svg><use xlink:href="#thumb-up-svg"></use></svg></i>"##.into()
        }
        SearchSort::Newest => r#"<i class="fa fa-refresh"></i>"#.into(),
        SearchSort::MostViewed => r#"<i class="fa fa-bar-chart"></i>"#.into(),
        SearchSort::MostCommented => r#"<i class="fa fa-comments"></i>"#.into(),
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
    let likes_svg = r##"<svg fill="#fff"><use xlink:href="#thumb-up-svg"></use></svg>"##;
    format!(
        concat!(
            r#"<div class="{cls}" itemscope="" itemtype="http://schema.org/ImageObject"> "#,
            r#"<a class="thumb-in" href="{url}" target="_blank" itemprop="url"> "#,
            r#"<div class="thumb-img"> <div class="video-preview"></div> "#,
            r#"<img class="thumb-cover" src="{lazy}" data-original="{thumb}" data-video="{preview}" alt="{title}" /> "#,
            r#"<div class="thumb-meta-top fx-row"><span class="ttime">{duration}</span></div> "#,
            r#"<div class="thumb-meta-bottom fx-row">"#,
            r#"<span class="tview"><i class="fa fa-eye"></i>{views}</span>"#,
            r#"<span class="tlike">{likes_svg}<span>{likes}</span></span>"#,
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
        likes_svg = likes_svg,
        likes = v.likes_label(),
    )
}

fn render_sort_links(
    page: u32,
    current: SearchSort,
    hd: HdFilter,
    listing: &ListingKind,
) -> String {
    let mut out = String::from(r#"<div class="select_sort">"#);
    for (label, key) in [
        ("Relevant", SearchSort::Relevant),
        ("Newest", SearchSort::Newest),
        ("Most Viewed", SearchSort::MostViewed),
        ("Most Commented", SearchSort::MostCommented),
    ] {
        let href = listing_path_with_query(listing, page, &SortKey::Search(key), hd);
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
