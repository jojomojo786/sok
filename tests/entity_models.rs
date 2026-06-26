//! Integration-style checks for pornstar/channel model exports and SQL contracts.

use sok::models::entities::{
    self, channel_profile_path, channel_thumb_url, pornstar_profile_path, pornstar_thumb_url,
    Channel, EntityListSort, Pornstar, SearchHelpChannel, SearchHelpPornstar, DEFAULT_MEDIA_CDN,
    ENTITY_SCHEMA_SQL,
};

#[test]
fn entity_module_exports_query_functions() {
    let _ = entities::list_pornstars_index;
    let _ = entities::list_channels_index;
    let _ = entities::pornstar_by_slug;
    let _ = entities::channel_by_slug;
    let _ = entities::search_pornstars_for_help;
    let _ = entities::search_channels_for_help;
    let _ = entities::videos_for_pornstar;
    let _ = entities::videos_for_channel;
}

#[test]
fn search_help_struct_fields_match_frontend_contract() {
    let star = SearchHelpPornstar {
        url_pornstar: pornstar_profile_path("angela-white"),
        orig_name: "Angela White".into(),
        thumb: pornstar_thumb_url(DEFAULT_MEDIA_CDN, "angela-white"),
    };
    assert!(star.url_pornstar.starts_with("/pornstar/"));
    assert!(star.thumb.contains("fox-images/pornstars"));

    let ch = SearchHelpChannel {
        url: channel_profile_path("brazzers"),
        orig_name: "Brazzers".into(),
        thumb: channel_thumb_url(DEFAULT_MEDIA_CDN, "brazzers"),
    };
    assert!(ch.url.starts_with("/channel/"));
    assert!(ch.thumb.contains("fox-images/channels"));
}

#[test]
fn profile_models_expose_media_and_counts() {
    let ps = Pornstar {
        id: 1,
        slug: "angela-white".into(),
        display_name: "Angela White".into(),
        thumb_path: String::new(),
        banner_path: Some("fox-images/pornstars/banners/angela-white.jpg".into()),
        avatar_path: None,
        bio: None,
        video_count: 10,
        verified: true,
        week_views: 100,
        created_at: None,
        updated_at: None,
    };
    assert_eq!(ps.profile_url(), "/pornstar/angela-white");
    assert!(ps
        .thumb_url(DEFAULT_MEDIA_CDN)
        .ends_with("angela-white.jpg"));
    assert!(ps.banner_url(DEFAULT_MEDIA_CDN).is_some());

    let ch = Channel {
        id: 2,
        slug: "brazzers".into(),
        title: "Brazzers".into(),
        thumb_path: String::new(),
        logo_path: Some("fox-images/channels/brazzers.jpg".into()),
        banner_path: None,
        bio: None,
        video_count: 99,
        network_name: None,
        week_views: 50,
        created_at: None,
        updated_at: None,
    };
    assert_eq!(ch.profile_url(), "/channel/brazzers");
    assert!(ch.logo_url(DEFAULT_MEDIA_CDN).contains("brazzers"));
}

#[test]
fn entity_queries_reference_required_tables_and_joins() {
    for token in [
        "pornstars",
        "channels",
        "pornstar_aliases",
        "channel_aliases",
        "video_pornstars",
        "week_views",
        "thumb_path",
        "banner_path",
        "avatar_path",
        "logo_path",
    ] {
        assert!(
            ENTITY_SCHEMA_SQL.contains(token),
            "missing schema token: {token}"
        );
    }
}

#[test]
fn sort_modes_cover_index_use_cases() {
    assert_eq!(EntityListSort::default(), EntityListSort::Trending);
    assert_ne!(
        EntityListSort::from_query_param(Some("name")),
        EntityListSort::VideoCountDesc
    );
}
