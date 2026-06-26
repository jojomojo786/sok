//! Fixture loader tests for deterministic dev/test catalog data (sok-replica.3.6).

use askama::Template;
use sok::fixtures::{
    load_catalog_seed, seed_category_cards, seed_home_thumbs, seed_pornstar_cards, DEV_CATALOG_SQL,
};
use sok::views::{
    categories_page_from_fixture_seed, CategoriesTemplate, IndexTemplate, PornstarsIndexView,
    PornstarsTemplate, RenderContext, SiteLayout,
};

#[test]
fn mirrored_seed_supports_non_empty_home_categories_pornstars_models() {
    let seed = load_catalog_seed().expect("catalog seed json");
    let home = seed_home_thumbs(&seed);
    let categories = seed_category_cards(&seed);
    let pornstars = seed_pornstar_cards(&seed);

    assert!(!home.is_empty(), "home grid");
    assert!(!categories.is_empty(), "categories grid");
    assert!(!pornstars.is_empty(), "pornstars grid");
}

#[test]
fn listing_templates_render_non_empty_html_from_fixture_backed_context() {
    let layout = SiteLayout::production();

    let home_seed = load_catalog_seed().expect("catalog seed json");
    let home_videos = seed_home_thumbs(&home_seed);
    let home_total = home_videos.len() as u64;
    let home_spec = sok::models::pagination::build_page_spec(
        sok::models::pagination::ListingKind::Home,
        1,
        home_total,
        &sok::models::pagination::ListingQueryParams::default(),
        None,
    )
    .expect("home page spec");
    let home_html = IndexTemplate {
        ctx: RenderContext::home_first_page(layout.clone()),
        page: sok::views::HomePageView::build(
            home_videos,
            &home_spec,
            home_total,
            &layout.site_base_url,
        ),
    }
    .render()
    .unwrap();
    assert!(home_html.contains("<h1>Top Trending Free Porn Videos</h1>"));
    assert!(
        home_html.matches("<!-- / thumb -->").count() >= 1,
        "fixture-backed home grid should be non-empty"
    );
    assert!(home_html.len() > 10_000);

    let cat_page = categories_page_from_fixture_seed(layout.clone());
    let categories_html = CategoriesTemplate {
        ctx: RenderContext::categories_index(layout.clone()),
        categories: cat_page.categories,
        top_tags: cat_page.top_tags,
        tag_preload_slugs: cat_page.tag_preload_slugs,
        top_pornstars: cat_page.top_pornstars,
    }
    .render()
    .unwrap();
    assert!(categories_html.contains("<h1>Porn Video Categories</h1>"));
    assert!(categories_html.len() > 10_000);

    let cards = seed_pornstar_cards(&load_catalog_seed().expect("seed"));
    let slug = cards[0].slug.clone();
    let q = sok::models::pagination::ListingQueryParams::default();
    let (_, meta) = sok::models::pagination::page_request(
        sok::models::pagination::ListingKind::EntityIndex(
            sok::models::pagination::EntityIndexKind::Pornstars,
        ),
        None,
        &q,
        cards.len() as u64,
        Some(&layout.site_base_url),
    )
    .unwrap();
    let pornstars_html = PornstarsTemplate {
        ctx: RenderContext::pornstars_index(layout.clone(), &meta),
        pornstars: PornstarsIndexView::build(
            cards.into_iter().take(48).collect(),
            &meta,
            &sok::models::pagination::SortKey::Entity(
                sok::models::pagination::EntitySortKey::Trending,
            ),
            layout.media_cdn.as_str(),
        ),
    }
    .render()
    .unwrap();
    assert!(pornstars_html.contains("<h1>Top Trending Pornstars</h1>"));
    assert!(pornstars_html.contains(&format!("/pornstar/{slug}")));
    assert!(pornstars_html.len() > 5_000);
}

#[test]
fn sql_seed_file_matches_json_fixture_slugs() {
    let seed = load_catalog_seed().expect("catalog seed json");
    assert!(DEV_CATALOG_SQL.contains(&seed.videos[0].slug));
    assert!(DEV_CATALOG_SQL.contains(&seed.categories[0].slug));
    assert!(DEV_CATALOG_SQL.contains(&seed.pornstars[0].slug));
}
