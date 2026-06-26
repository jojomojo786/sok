//! Path-based integration checks for taxonomy exports and page contracts.

use sok::models::taxonomy::{
    self, CategoryCard, CatsTagsSearchItem, CatsTagsSearchResponse, ListingSlugKind, ListingSort,
    SlugListingQuery, CATEGORY_THUMB_CDN_PREFIX,
};

#[test]
fn taxonomy_module_exports_page_query_functions() {
    let _ = taxonomy::list_categories_for_index;
    let _ = taxonomy::search_categories_and_tags;
    let _ = taxonomy::get_category_by_slug;
    let _ = taxonomy::get_tag_by_slug;
    let _ = taxonomy::resolve_listing_slug;
    let _ = taxonomy::list_video_ids_for_category;
    let _ = taxonomy::list_video_ids_for_tag;
    let _ = taxonomy::list_top_viewed_tags;
}

#[test]
fn categories_index_card_shape_matches_docs() {
    let card = CategoryCard {
        slug: "milf".into(),
        title: "MILF".into(),
        thumb_url: format!("{CATEGORY_THUMB_CDN_PREFIX}milf.jpg"),
        video_count: 100,
        listing_url: "/milf".into(),
    };
    assert_eq!(card.listing_url, "/milf");
    assert!(card.thumb_url.contains("fox-images/categories"));
}

#[test]
fn ajax_search_json_shape_matches_categories_doc() {
    let resp = CatsTagsSearchResponse {
        search_text: "ana".into(),
        items: vec![CatsTagsSearchItem {
            id: "42".into(),
            name: "Anal".into(),
            url: "/anal".into(),
        }],
    };
    let v: serde_json::Value = serde_json::to_value(&resp).unwrap();
    assert_eq!(v["search_text"], "ana");
    assert_eq!(v["items"][0]["id"], "42");
    assert_eq!(v["items"][0]["url"], "/anal");
}

#[test]
fn slug_listing_query_wires_sort_and_hd_from_docs() {
    let q = SlugListingQuery {
        slug: "milf".into(),
        sort: ListingSort::from_query(Some("mv")),
        hd_only: true,
    };
    assert_eq!(q.sort, ListingSort::MostViewed);
    assert!(q.hd_only);
    assert_eq!(ListingSlugKind::Category, ListingSlugKind::Category);
}
