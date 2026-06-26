use sok::models::pagination::{
    build_page_nav, build_page_spec, listing_page_path, page_request, parse_page_number,
    EntityIndexKind, EntitySortKey, HdFilter, ListingKind, ListingQueryParams, PageNavItem,
    PaginationError, SortKey, DEFAULT_HOME_VIDEO_PER_PAGE,
};
use sok::models::taxonomy::ListingSort;

#[test]
fn invalid_page_rejects_non_numeric() {
    let err = parse_page_number("milf").unwrap_err();
    assert!(matches!(err, PaginationError::InvalidPage(_)));
}

#[test]
fn page_spec_clamps_to_total_pages() {
    let q = ListingQueryParams::default();
    let err = build_page_spec(
        ListingKind::Home,
        100,
        10,
        &q,
        Some(DEFAULT_HOME_VIDEO_PER_PAGE),
    )
    .unwrap_err();
    assert!(matches!(err, PaginationError::PageOutOfRange { .. }));
}

#[test]
fn entity_profile_channel_pagination_path() {
    let listing = ListingKind::EntityProfile(
        sok::models::pagination::EntityProfileKind::Channel,
        "brazzers".into(),
    );
    assert_eq!(listing_page_path(&listing, 2), "/channel/brazzers/2");
}

#[test]
fn pornstars_entity_sort_query() {
    let q = ListingQueryParams {
        sort: Some("videocount".into()),
        hd: None,
        page: None,
    };
    let listing = ListingKind::EntityIndex(EntityIndexKind::Pornstars);
    let spec = build_page_spec(listing, 1, 500, &q, None).unwrap();
    assert_eq!(spec.sort, SortKey::Entity(EntitySortKey::VideoCount));
}

#[test]
fn integration_home_pagination_meta_offsets() {
    let q = ListingQueryParams::default();
    let (spec, meta) = page_request(ListingKind::Home, Some("5"), &q, 54 * 20, None).unwrap();
    assert_eq!(spec.offset, 54 * 4);
    assert_eq!(meta.page, 5);
    assert_eq!(meta.total_pages, 20);
    assert!(meta.has_previous);
    assert!(meta.has_next);
}

#[test]
fn page_nav_includes_last_page_when_far() {
    let nav = build_page_nav(
        &ListingKind::EntityIndex(EntityIndexKind::Pornstars),
        1,
        83,
        &SortKey::Entity(EntitySortKey::Trending),
        HdFilter::All,
    );
    assert!(nav
        .iter()
        .any(|i| matches!(i, PageNavItem::Link { page: 83, .. })));
}

#[test]
fn video_sort_uses_taxonomy_listing_sort() {
    let q = ListingQueryParams {
        sort: Some("mc".into()),
        hd: None,
        page: None,
    };
    let sort = q.parse_sort_for_listing(&ListingKind::CategorySlug {
        slug: "anal".into(),
    });
    assert_eq!(sort, SortKey::Video(ListingSort::MostCommented));
}
