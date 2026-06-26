//! Shared Askama render context: layout defaults, asset paths, and per-page SEO meta.

use crate::config::{self, Config};
use crate::models::pagination::{absolute_url, DEFAULT_SITE_BASE_URL};

pub use crate::config::DEFAULT_MEDIA_CDN;

/// Static app mounts and adult-catalog CDN paths used by templates and JS globals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetPaths {
    pub static_root: String,
    pub fox_tpl_root: String,
    pub media_cdn: String,
    pub thumbs_videos_url: String,
    pub thumbs_videos_dir: String,
    pub video_path_segment: String,
}

impl AssetPaths {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            static_root: cfg.static_root.clone(),
            fox_tpl_root: cfg.fox_tpl_root.clone(),
            media_cdn: cfg.media_cdn.clone(),
            thumbs_videos_url: cfg.thumbs_videos_url(),
            thumbs_videos_dir: cfg.thumbs_videos_dir.clone(),
            video_path_segment: cfg.video_path_segment.clone(),
        }
    }

    pub fn static_fox_tpl_directory(&self) -> String {
        format!("{}/fox-tpl", self.static_root.trim_end_matches('/'))
    }

    pub fn fox_tpl_directory(&self) -> String {
        self.fox_tpl_root.trim_end_matches('/').to_string()
    }
}

impl Default for AssetPaths {
    fn default() -> Self {
        Self::from_config(&Config::asset_defaults())
    }
}

/// Theme and document defaults mirrored from production HTML shells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeDefaults {
    pub html_lang: &'static str,
    pub data_theme: &'static str,
    pub og_prefix: &'static str,
    pub msapplication_tile_color: &'static str,
    pub theme_color: &'static str,
    pub rta_rating: &'static str,
    pub og_type: &'static str,
    pub og_site_name: &'static str,
    pub og_image: &'static str,
}

impl Default for ThemeDefaults {
    fn default() -> Self {
        Self {
            html_lang: "en",
            data_theme: "dark",
            og_prefix: "og: http://ogp.me/ns#",
            msapplication_tile_color: "#da532c",
            theme_color: "#ffffff",
            rta_rating: "RTA-5042-1996-1400-1577-RTA",
            og_type: "website",
            og_site_name: "pornsok.com",
            og_image: "https://c.foxporn.tv/",
        }
    }
}

/// Site-wide layout data shared across listing templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteLayout {
    pub site_base_url: String,
    /// Configurable CDN host (posters, previews, download placeholders).
    pub media_cdn: String,
    pub assets: AssetPaths,
    pub theme: ThemeDefaults,
    pub copyright_year: u16,
}

impl SiteLayout {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            site_base_url: DEFAULT_SITE_BASE_URL.to_string(),
            media_cdn: cfg.media_cdn.clone(),
            assets: AssetPaths::from_config(cfg),
            theme: ThemeDefaults::default(),
            copyright_year: 2026,
        }
    }

    pub fn production() -> Self {
        Self::from_config(&Config::asset_defaults())
    }

    pub fn with_site_base(mut self, site_base_url: impl Into<String>) -> Self {
        self.site_base_url = site_base_url.into();
        self
    }

    pub fn with_media_cdn(mut self, media_cdn: impl Into<String>) -> Self {
        let media_cdn = media_cdn.into();
        self.media_cdn = media_cdn.clone();
        self.assets.media_cdn = media_cdn.clone();
        self.assets.thumbs_videos_url =
            config::media_url(&media_cdn, &self.assets.thumbs_videos_dir);
        self
    }
}

/// Per-page SEO and visible heading fields (title, canonical, rel prev/next, Open Graph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub canonical_path: String,
    pub h1: String,
    pub rel_prev_href: Option<String>,
    pub rel_next_href: Option<String>,
    pub og_title: String,
    pub og_description: String,
    pub og_url: String,
}

impl PageMeta {
    pub fn canonical_href(&self, site_base: &str) -> String {
        absolute_url(site_base, &self.canonical_path)
    }

    pub fn og_url_absolute(&self, site_base: &str) -> String {
        if self.og_url.starts_with("http://") || self.og_url.starts_with("https://") {
            self.og_url.clone()
        } else {
            absolute_url(site_base, &self.og_url)
        }
    }
}

/// Combined layout + page meta passed into Askama templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderContext {
    pub layout: SiteLayout,
    pub page: PageMeta,
}

impl RenderContext {
    pub fn new(layout: SiteLayout, page: PageMeta) -> Self {
        Self { layout, page }
    }

    pub fn canonical_href(&self) -> String {
        self.page.canonical_href(&self.layout.site_base_url)
    }

    pub fn og_url_absolute(&self) -> String {
        self.page.og_url_absolute(&self.layout.site_base_url)
    }

    pub fn boot_script(&self) -> String {
        crate::views::PlayerBootGlobals::listing_page(&self.layout.assets, &self.layout.media_cdn)
            .to_inline_script()
    }

    pub fn home_boot_script(&self) -> String {
        crate::views::PlayerBootGlobals::home_listing_page(
            &self.layout.assets,
            &self.layout.media_cdn,
        )
        .to_inline_script()
    }

    pub fn home_first_page(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let description = "PornsOK features new porn videos every day for free. Come to browse the hottest XXX movies from the sexiest pornstars and models. Nothing but the most fantastic porn movies can be found here!";
        let title = "Free Porn Videos & Hot 🌶️ Sex Movies | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/".into(),
            h1: "Top Trending Free Porn Videos".into(),
            rel_prev_href: None,
            rel_next_href: Some(absolute_url(&site, "/2")),
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/"),
        };
        Self::new(layout, page)
    }

    /// Page-aware home meta (canonical + rel prev/next) driven by pagination metadata.
    pub fn home_listing(
        layout: SiteLayout,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let description = "PornsOK features new porn videos every day for free. Come to browse the hottest XXX movies from the sexiest pornstars and models. Nothing but the most fantastic porn movies can be found here!";
        let title = "Free Porn Videos & Hot 🌶️ Sex Movies | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: meta.canonical_path.clone(),
            h1: "Top Trending Free Porn Videos".into(),
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn categories_index(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let description = "PornsOK features an ultimate selection of porn video categories, such as MILF, Anal, Teens, and Lesbians. Visit PornsOK.com and find your favorite sex category right away!";
        let title = "Porn Categories: Hottest Sex Niches | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/categories".into(),
            h1: "Porn Video Categories".into(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/categories"),
        };
        Self::new(layout, page)
    }

    pub fn pornstars_index_first_page(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let description = "PornsOK brings you the most popular pornstars and models for free. Find your favorite actresses that are naked for you 365 days a year!";
        let title = "Best Pornstars and Models in Free Porn Videos | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/pornstars".into(),
            h1: "Top Trending Pornstars".into(),
            rel_prev_href: None,
            rel_next_href: Some(absolute_url(&site, "/pornstars/2")),
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/pornstars"),
        };
        Self::new(layout, page)
    }

    pub fn pornstars_index(
        layout: SiteLayout,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let description = "PornsOK brings you the most popular pornstars and models for free. Find your favorite actresses that are naked for you 365 days a year!";
        let title = "Best Pornstars and Models in Free Porn Videos | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: meta.canonical_path.clone(),
            h1: "Top Trending Pornstars".into(),
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn channels_index_first_page(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let description = "Discover a complete list of porn channels on PornsOK.com! Find free porn movies from your favorite XXX studios and porn labels right here.";
        let title = "Free Porn Channels: List of Best Sex Channels | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/channels".into(),
            h1: "Top Trending Porn Channels".into(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/channels"),
        };
        Self::new(layout, page)
    }

    pub fn channels_index(
        layout: SiteLayout,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let description = "Discover a complete list of porn channels on PornsOK.com! Find free porn movies from your favorite XXX studios and porn labels right here.";
        let title = "Free Porn Channels: List of Best Sex Channels | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: meta.canonical_path.clone(),
            h1: "Top Trending Porn Channels".into(),
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn entity_profile(
        layout: SiteLayout,
        header: &crate::models::entities::EntityProfileHeader,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let page = PageMeta {
            title: header.title.clone(),
            description: header.description.clone(),
            canonical_path: meta.canonical_path.clone(),
            h1: header.h1.clone(),
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: header.h1.clone(),
            og_description: header.description.clone(),
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn slug_listing(
        layout: SiteLayout,
        header: &crate::models::taxonomy::TaxonomyListingHeader,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let name = header.display_name.clone();
        let description = header
            .description
            .clone()
            .filter(|d| !d.trim().is_empty())
            .unwrap_or_else(|| {
                format!(
                    "Watch the latest {name} porn videos for free on PornsOK.com. \
                     Stream new {name} XXX scenes in HD every day."
                )
            });
        let title = format!("{name} Porn Videos: Free {name} Sex Movies | PornsOK.com");
        let page = PageMeta {
            title,
            description: description.clone(),
            canonical_path: meta.canonical_path.clone(),
            h1: header.h1.clone(),
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: format!("{name} - Latest Porn Scenes"),
            og_description: description,
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn search_listing(
        layout: SiteLayout,
        display_term: &str,
        meta: &crate::models::pagination::PaginationMeta,
    ) -> Self {
        let site = layout.site_base_url.clone();
        let title = crate::models::search::search_page_title(display_term);
        let description = crate::models::search::search_meta_description(display_term);
        let h1 = crate::models::search::search_h1(display_term);
        let page = PageMeta {
            title: title.clone(),
            description: description.clone(),
            canonical_path: meta.canonical_path.clone(),
            h1,
            rel_prev_href: meta.rel_prev.clone(),
            rel_next_href: meta.rel_next.clone(),
            og_title: title,
            og_description: description,
            og_url: absolute_url(&site, &meta.canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn tags_hub(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let description = "Browse all porn tags on PornsOK.com. Discover thousands of free XXX videos organized by tag and find exactly the scenes you are looking for.";
        let title = "Porn Tags: Browse All Sex Video Tags | PornsOK.com";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/tags".into(),
            h1: "Porn Video Tags".into(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/tags"),
        };
        Self::new(layout, page)
    }

    pub fn not_found_page(layout: SiteLayout, slug: &str) -> Self {
        let site = layout.site_base_url.clone();
        let canonical_path = format!("/{slug}");
        let title = "Page not found | PornsOK.com";
        let description = "The page you requested is not available. Try the home page or browse categories for free porn videos.";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: canonical_path.clone(),
            h1: "Page not found".into(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, &canonical_path),
        };
        Self::new(layout, page)
    }

    pub fn internal_error_page(layout: SiteLayout) -> Self {
        let site = layout.site_base_url.clone();
        let title = "Something went wrong | PornsOK.com";
        let description = "Something went wrong on our side. Please try again in a moment.";
        let page = PageMeta {
            title: title.into(),
            description: description.into(),
            canonical_path: "/".into(),
            h1: "Something went wrong".into(),
            rel_prev_href: None,
            rel_next_href: None,
            og_title: title.into(),
            og_description: description.into(),
            og_url: absolute_url(&site, "/"),
        };
        Self::new(layout, page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::{CategoriesTemplate, IndexTemplate, PornstarsIndexView, PornstarsTemplate};
    use crate::views::{ChannelsIndexView, ChannelsTemplate};
    use askama::Template;

    fn layout() -> SiteLayout {
        SiteLayout::production()
    }

    fn sample_home_page_view() -> crate::views::HomePageView {
        use crate::fixtures::{load_catalog_seed, seed_home_thumbs};
        use crate::models::pagination::{build_page_spec, ListingKind, ListingQueryParams};
        let seed = load_catalog_seed().unwrap();
        let videos = seed_home_thumbs(&seed);
        let total = videos.len() as u64;
        let spec = build_page_spec(
            ListingKind::Home,
            1,
            total,
            &ListingQueryParams::default(),
            None,
        )
        .unwrap();
        crate::views::HomePageView::build(videos, &spec, total, &layout().site_base_url)
    }

    #[test]
    fn home_boot_script_uses_local_static_directory() {
        let ctx = RenderContext::home_first_page(layout());
        let script = ctx.home_boot_script();
        assert!(script.contains("isTHUMBS_OR_PLAYER = true"));
        assert!(script.contains("directory = \"/static/fox-tpl\""));
        assert!(script.contains("thumbs_path = \"https://c.foxporn.tv/fox-images/videos\""));
    }

    #[test]
    fn listing_boot_script_uses_local_fox_tpl_directory() {
        let ctx = RenderContext::categories_index(layout());
        let script = ctx.boot_script();
        assert!(script.contains("isTHUMBS_OR_PLAYER = false"));
        assert!(script.contains("directory = \"/fox-tpl\""));
    }

    #[test]
    fn configurable_media_cdn_updates_listing_boot_thumbs_path() {
        let layout = layout().with_media_cdn("https://cdn.example.com/");
        let ctx = RenderContext::categories_index(layout);
        let script = ctx.boot_script();
        assert!(script.contains("thumbs_path = \"https://cdn.example.com/fox-images/videos\""));
    }

    #[test]
    fn layout_defaults_expose_asset_and_theme_paths() {
        let ctx = RenderContext::home_first_page(layout());
        assert_eq!(ctx.layout.assets.static_root, "/static");
        assert_eq!(ctx.layout.assets.fox_tpl_root, "/fox-tpl");
        assert_eq!(ctx.layout.theme.data_theme, "dark");
        assert_eq!(ctx.layout.theme.og_site_name, "pornsok.com");
    }

    #[test]
    fn home_meta_matches_static_mirror() {
        let ctx = RenderContext::home_first_page(layout());
        let site = &ctx.layout.site_base_url;
        assert_eq!(
            ctx.page.title,
            "Free Porn Videos & Hot 🌶️ Sex Movies | PornsOK.com"
        );
        assert_eq!(ctx.page.canonical_href(site), "https://pornsok.com/");
        assert_eq!(ctx.page.h1, "Top Trending Free Porn Videos");
        assert_eq!(
            ctx.page.rel_next_href.as_deref(),
            Some("https://pornsok.com/2")
        );
    }

    #[test]
    fn categories_meta_matches_static_mirror() {
        let ctx = RenderContext::categories_index(layout());
        let site = &ctx.layout.site_base_url;
        assert_eq!(
            ctx.page.title,
            "Porn Categories: Hottest Sex Niches | PornsOK.com"
        );
        assert_eq!(
            ctx.page.canonical_href(site),
            "https://pornsok.com/categories"
        );
        assert_eq!(ctx.page.h1, "Porn Video Categories");
    }

    #[test]
    fn pornstars_meta_matches_static_mirror() {
        let ctx = RenderContext::pornstars_index_first_page(layout());
        let site = &ctx.layout.site_base_url;
        assert_eq!(
            ctx.page.title,
            "Best Pornstars and Models in Free Porn Videos | PornsOK.com"
        );
        assert_eq!(ctx.page.h1, "Top Trending Pornstars");
        assert_eq!(
            ctx.page.canonical_href(site),
            "https://pornsok.com/pornstars"
        );
        assert_eq!(
            ctx.page.rel_next_href.as_deref(),
            Some("https://pornsok.com/pornstars/2")
        );
    }

    #[test]
    fn index_template_render_includes_meta() {
        let html = IndexTemplate {
            ctx: RenderContext::home_first_page(layout()),
            page: sample_home_page_view(),
        }
        .render()
        .unwrap();
        assert!(html.contains("Free Porn Videos") && html.contains("PornsOK.com</title>"));
        assert!(html.contains(r#"canonical" href="https://pornsok.com/""#));
        assert!(html.contains("<h1>Top Trending Free Porn Videos</h1>"));

        let cat_page = crate::views::categories_page_from_fixture_seed(layout());
        let html = CategoriesTemplate {
            ctx: RenderContext::categories_index(layout()),
            categories: cat_page.categories,
            top_tags: cat_page.top_tags,
            tag_preload_slugs: cat_page.tag_preload_slugs,
            top_pornstars: cat_page.top_pornstars,
        }
        .render()
        .unwrap();
        assert!(html.contains("Porn Categories: Hottest Sex Niches"));
        assert!(html.contains(r#"canonical" href="https://pornsok.com/categories""#));
        assert!(html.contains("<h1>Porn Video Categories</h1>"));

        let html = PornstarsTemplate {
            ctx: RenderContext::pornstars_index_first_page(layout()),
            pornstars: PornstarsIndexView::default(),
        }
        .render()
        .unwrap();
        assert!(html.contains("Best Pornstars and Models in Free Porn Videos"));
        assert!(html.contains(r#"canonical" href="https://pornsok.com/pornstars""#));
        assert!(html.contains("<h1>Top Trending Pornstars</h1>"));
    }

    #[test]
    fn pornstars_template_renders_fixture_card_slug() {
        use crate::fixtures::{load_catalog_seed, seed_pornstar_cards};

        let layout = layout();
        let seed = load_catalog_seed().expect("catalog seed");
        let cards = seed_pornstar_cards(&seed);
        let slug = cards[0].slug.clone();
        let q = crate::models::pagination::ListingQueryParams::default();
        let (_, meta) = crate::models::pagination::page_request(
            crate::models::pagination::ListingKind::EntityIndex(
                crate::models::pagination::EntityIndexKind::Pornstars,
            ),
            None,
            &q,
            cards.len() as u64,
            Some(&layout.site_base_url),
        )
        .unwrap();
        let view = PornstarsIndexView::build(
            cards.into_iter().take(48).collect(),
            &meta,
            &crate::models::pagination::SortKey::Entity(
                crate::models::pagination::EntitySortKey::Trending,
            ),
            &layout.media_cdn,
        );
        let html = PornstarsTemplate {
            ctx: RenderContext::pornstars_index(layout, &meta),
            pornstars: view,
        }
        .render()
        .unwrap();
        assert!(html.contains(&format!("/pornstar/{slug}")));
        assert!(html.contains("all_pornstars"));
        assert!(html.contains("search_type = 'pstars'"));
    }

    #[test]
    fn channels_index_meta_matches_docs() {
        let html = ChannelsTemplate {
            ctx: RenderContext::channels_index_first_page(layout()),
            channels: ChannelsIndexView::default(),
        }
        .render()
        .unwrap();
        assert!(html.contains("Free Porn Channels: List of Best Sex Channels"));
        assert!(html.contains(r#"canonical" href="https://pornsok.com/channels""#));
        assert!(html.contains("<h1>Top Trending Porn Channels</h1>"));
        assert!(html.contains("search_type = 'channels'"));
        assert!(html.contains("Search by all channels and studios"));
    }

    #[test]
    fn channels_template_renders_fixture_card_slug() {
        use crate::fixtures::{load_catalog_seed, seed_channel_cards};

        let layout = layout();
        let seed = load_catalog_seed().expect("catalog seed");
        let cards = seed_channel_cards(&seed);
        let slug = cards[0].slug.clone();
        let q = crate::models::pagination::ListingQueryParams::default();
        let (_, meta) = crate::models::pagination::page_request(
            crate::models::pagination::ListingKind::EntityIndex(
                crate::models::pagination::EntityIndexKind::Channels,
            ),
            None,
            &q,
            cards.len() as u64,
            Some(&layout.site_base_url),
        )
        .unwrap();
        let view = ChannelsIndexView::build(
            cards.into_iter().take(48).collect(),
            &meta,
            &crate::models::pagination::SortKey::Entity(
                crate::models::pagination::EntitySortKey::Trending,
            ),
            &layout.media_cdn,
        );
        let html = ChannelsTemplate {
            ctx: RenderContext::channels_index(layout, &meta),
            channels: view,
        }
        .render()
        .unwrap();
        assert!(html.contains(&format!("/channel/{slug}")));
        assert!(html.contains("all_pornstars"));
        assert!(html.contains("thumb cat"));
    }

    #[test]
    fn theme_defaults_expose_rta_label_for_templates() {
        let layout = layout();
        assert_eq!(layout.theme.rta_rating, "RTA-5042-1996-1400-1577-RTA");
    }

    #[test]
    fn search_listing_meta_matches_docs_pattern() {
        use crate::models::pagination::{page_request, ListingKind, ListingQueryParams};
        let layout = layout();
        let q = ListingQueryParams::default();
        let (_, meta) = page_request(
            ListingKind::Search {
                query_slug: "test".into(),
            },
            None,
            &q,
            10,
            Some(&layout.site_base_url),
        )
        .unwrap();
        let ctx = RenderContext::search_listing(layout.clone(), "Test", &meta);
        assert_eq!(
            ctx.page.title,
            "Test Porn Videos & Sex Scenes | PornsOK.com"
        );
        assert_eq!(ctx.page.canonical_path, "/videos/test");
        assert_eq!(ctx.page.og_title, ctx.page.title);
        assert_eq!(ctx.page.og_description, ctx.page.description);
        assert_eq!(ctx.og_url_absolute(), "https://pornsok.com/videos/test");
    }

    #[test]
    fn slug_listing_meta_falls_back_description_when_header_empty() {
        use crate::models::pagination::{page_request, ListingKind, ListingQueryParams};
        use crate::models::taxonomy::CategoryRow;
        let layout = layout();
        let header = CategoryRow {
            id: 2,
            slug: "teen".into(),
            display_name: "Teen".into(),
            description: None,
            thumb_url: None,
            video_count: 3,
            intro_html: None,
            sort_order: 0,
            is_active: true,
        }
        .to_listing_header();
        let q = ListingQueryParams::default();
        let (_, meta) = page_request(
            ListingKind::CategorySlug {
                slug: "teen".into(),
            },
            None,
            &q,
            5,
            Some(&layout.site_base_url),
        )
        .unwrap();
        let ctx = RenderContext::slug_listing(layout, &header, &meta);
        assert_eq!(
            ctx.page.title,
            "Teen Porn Videos: Free Teen Sex Movies | PornsOK.com"
        );
        assert!(ctx.page.description.contains("Watch the latest Teen porn"));
        assert_eq!(ctx.page.h1, "Teen - Latest Porn Scenes");
    }

    #[test]
    fn tags_hub_meta_matches_docs() {
        let ctx = RenderContext::tags_hub(layout());
        assert_eq!(ctx.page.canonical_path, "/tags");
        assert_eq!(ctx.page.h1, "Porn Video Tags");
        assert!(ctx.page.description.contains("Browse all porn tags"));
    }
}
