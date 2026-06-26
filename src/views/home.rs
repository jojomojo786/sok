//! Home page view model: primary video grid, filters, pagination, and SEO copy.

use crate::models::pagination::{
    build_page_nav, listing_path_with_query, pagination_meta, total_pages, HdFilter, ListingKind,
    PageSpec, PaginationMeta, SortKey,
};
use crate::models::taxonomy::ListingSort;
use crate::models::video::VideoThumb;

/// SEO intro paragraph shown above the grid (`.toptext`).
pub const HOME_SEO_INTRO: &str = "Horny guys looking for the best free porn video don't need to look anywhere else. At PornsOK.com, the porno videos are just made for you! Just check out the huge variety of videos in our collection. Check out porno videos of your favorite sex position on this amazing porn website. From mature ladies to amateur babes, find everything you need to blow your mind all in one porn tube. One thing is for sure, PornsOK.com is the only porn website that you'll ever need.";

/// SEO footer paragraph shown below the grid (`.desc-text`).
pub const HOME_SEO_FOOTER: &str = "Masturbating has never been this fun! With the free porn videos at PornsOK, enjoy porn in every category, all day every day. No matter where you are, you can never have too much of porno videos. Anytime you feel like blowing your load, just hop on to this awesome porn site and enjoy some free porn videos of your choosing. There isn't a single category that doesn't have a free porn video against it. There is no shortage of hot, steamy videos from the most beautiful babes on this porn tube. So, why wait? Join today for some action. Who knows? Maybe you'll get to enjoy some premium videos on the site.";

/// 1x1 transparent GIF used as the lazy-load placeholder `src` on `.thumb-cover`.
pub const THUMB_LAZY_PLACEHOLDER: &str =
    "data:image/gif;base64,R0lGODlhAQABAJAAAAAAAAAAACH5BAEUAAAALAAAAAABAAEAAAICRAEAOw==";

/// All/HD toggle and sort links for the `.filter-section` controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeFilterView {
    pub all_href: String,
    pub hd_href: String,
    pub newest_href: String,
    pub most_viewed_href: String,
    pub most_commented_href: String,
    pub all_active: bool,
    pub hd_active: bool,
    pub newest_selected: bool,
    pub most_viewed_selected: bool,
    pub most_commented_selected: bool,
}

impl HomeFilterView {
    fn build(sort: ListingSort, hd: HdFilter) -> Self {
        let listing = ListingKind::Home;
        let href = |sort: SortKey, hd: HdFilter| listing_path_with_query(&listing, 1, &sort, hd);
        HomeFilterView {
            all_href: href(SortKey::Video(sort), HdFilter::All),
            hd_href: href(SortKey::Video(sort), HdFilter::HdOnly),
            newest_href: href(SortKey::Video(ListingSort::Latest), hd),
            most_viewed_href: href(SortKey::Video(ListingSort::MostViewed), hd),
            most_commented_href: href(SortKey::Video(ListingSort::MostCommented), hd),
            all_active: hd == HdFilter::All,
            hd_active: hd == HdFilter::HdOnly,
            newest_selected: sort == ListingSort::Latest,
            most_viewed_selected: sort == ListingSort::MostViewed,
            most_commented_selected: sort == ListingSort::MostCommented,
        }
    }
}

/// One rendered item in the `.page_nav` pagination control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HomePaginationItem {
    Current(u32),
    Link { href: String, label: String },
    Ellipsis,
    Previous { href: String },
    Next { href: String },
}

impl HomePaginationItem {
    pub fn from_page_nav(items: Vec<crate::models::pagination::PageNavItem>) -> Vec<Self> {
        use crate::models::pagination::PageNavItem;
        items
            .into_iter()
            .map(|item| match item {
                PageNavItem::Current(p) => Self::Current(p),
                PageNavItem::Link { href, page } => Self::Link {
                    href,
                    label: page.to_string(),
                },
                PageNavItem::Ellipsis => Self::Ellipsis,
                PageNavItem::Previous { href, .. } => Self::Previous { href },
                PageNavItem::Next { href, .. } => Self::Next { href },
            })
            .collect()
    }

    // Askama-friendly variant accessors for `{% match %}`-free templates.
    pub fn is_current(&self) -> bool {
        matches!(self, Self::Current(_))
    }
    pub fn is_ellipsis(&self) -> bool {
        matches!(self, Self::Ellipsis)
    }
    pub fn is_previous(&self) -> bool {
        matches!(self, Self::Previous { .. })
    }
    pub fn is_next(&self) -> bool {
        matches!(self, Self::Next { .. })
    }
    pub fn is_link(&self) -> bool {
        matches!(self, Self::Link { .. })
    }
    pub fn href(&self) -> &str {
        match self {
            Self::Link { href, .. } | Self::Previous { href } | Self::Next { href } => href,
            _ => "",
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::Current(p) => p.to_string(),
            Self::Link { label, .. } => label.clone(),
            _ => String::new(),
        }
    }
}

/// Full data-driven model for the home page primary grid.
#[derive(Debug, Clone)]
pub struct HomePageView {
    pub videos: Vec<VideoThumb>,
    pub pagination: Vec<HomePaginationItem>,
    pub filters: HomeFilterView,
    pub seo_intro: String,
    pub seo_footer: String,
    pub thumb_placeholder: String,
}

impl HomePageView {
    pub fn has_videos(&self) -> bool {
        !self.videos.is_empty()
    }

    /// Build the view from a fetched page of thumbs and the resolved page spec.
    pub fn build(
        videos: Vec<VideoThumb>,
        spec: &PageSpec,
        total_items: u64,
        site_base: &str,
    ) -> Self {
        let sort = match &spec.sort {
            SortKey::Video(s) => *s,
            _ => ListingSort::Latest,
        };
        let meta: PaginationMeta = pagination_meta(spec, total_items, site_base);
        let total = total_pages(total_items, spec.per_page);
        let nav = build_page_nav(&ListingKind::Home, meta.page, total, &spec.sort, spec.hd);
        HomePageView {
            videos,
            pagination: HomePaginationItem::from_page_nav(nav),
            filters: HomeFilterView::build(sort, spec.hd),
            seo_intro: HOME_SEO_INTRO.to_string(),
            seo_footer: HOME_SEO_FOOTER.to_string(),
            thumb_placeholder: THUMB_LAZY_PLACEHOLDER.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pagination::{build_page_spec, ListingKind, ListingQueryParams};
    use crate::models::video::VideoThumb;

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

    fn spec_for(query: &str, total: u64, page: u32) -> crate::models::pagination::PageSpec {
        let q: ListingQueryParams = serde_urlencoded::from_str(query).unwrap_or_default();
        build_page_spec(ListingKind::Home, page, total, &q, None).unwrap()
    }

    #[test]
    fn home_view_first_page_default_filters_and_grid() {
        let spec = spec_for("", 540, 1);
        let view = HomePageView::build(vec![thumb(1), thumb(2)], &spec, 540, "https://pornsok.com");
        assert!(view.has_videos());
        assert_eq!(view.videos.len(), 2);
        // All active, Newest selected by default.
        assert!(view.filters.all_active);
        assert!(!view.filters.hd_active);
        assert!(view.filters.newest_selected);
        assert!(!view.filters.most_viewed_selected);
        assert_eq!(view.filters.hd_href, "/?hd=1");
        assert_eq!(view.filters.most_viewed_href, "/?sort=mv");
        assert_eq!(view.filters.most_commented_href, "/?sort=mc");
        assert!(view.seo_intro.starts_with("Horny guys"));
        assert!(view.seo_footer.starts_with("Masturbating"));
    }

    #[test]
    fn home_view_pagination_links_to_page_two_with_home_prefix() {
        let spec = spec_for("", 540, 1);
        let view = HomePageView::build(vec![thumb(1)], &spec, 540, "https://pornsok.com");
        let links: Vec<String> = view
            .pagination
            .iter()
            .filter(|i| i.is_link() || i.is_next())
            .map(|i| i.href().to_string())
            .collect();
        assert!(links.iter().any(|h| h == "/2"), "expected /2 in {links:?}");
        // Current page marker is page 1.
        assert!(view
            .pagination
            .iter()
            .any(|i| i.is_current() && i.label() == "1"));
    }

    #[test]
    fn home_view_sort_mv_marks_most_viewed_and_keeps_query() {
        let spec = spec_for("sort=mv", 540, 1);
        let view = HomePageView::build(vec![thumb(1)], &spec, 540, "https://pornsok.com");
        assert!(view.filters.most_viewed_selected);
        assert!(!view.filters.newest_selected);
        // Pagination preserves the sort query.
        let next = view.pagination.iter().find(|i| i.is_next());
        assert_eq!(
            next.map(|i| i.href().to_string()),
            Some("/2?sort=mv".into())
        );
    }

    #[test]
    fn home_view_hd_filter_active_and_href() {
        let spec = spec_for("hd=1", 540, 1);
        let view = HomePageView::build(vec![thumb(1)], &spec, 540, "https://pornsok.com");
        assert!(view.filters.hd_active);
        assert!(!view.filters.all_active);
        assert_eq!(view.filters.all_href, "/");
    }
}
