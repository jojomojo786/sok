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

/// Regression guard for sok-replica.3.8: the live Aiven catalog was created
/// from `0001_catalog_schema.sql` (legacy `display_name` / `thumb_url` /
/// `view_count`), so header autocomplete failed with `Unknown column
/// 'p.thumb_path'`. The additive `0002_align_catalog_search_thumbs.sql`
/// migration must add (and backfill) every column the search queries select.
#[test]
fn alignment_migration_adds_search_thumbnail_columns() {
    let migration = include_str!("../migrations/0002_align_catalog_search_thumbs.sql");

    // Columns the AJAX search/autocomplete queries depend on.
    for token in [
        "ALTER TABLE pornstars ADD COLUMN thumb_path",
        "ALTER TABLE pornstars ADD COLUMN week_views",
        "ALTER TABLE channels ADD COLUMN title",
        "ALTER TABLE channels ADD COLUMN thumb_path",
        "ALTER TABLE channels ADD COLUMN week_views",
        "ALTER TABLE videos ADD COLUMN views",
        "ALTER TABLE videos ADD COLUMN wide_thumb",
        "ALTER TABLE videos ADD COLUMN status",
    ] {
        assert!(
            migration.contains(token),
            "0002 migration missing column alignment: {token}"
        );
    }

    // Each column add is guarded so a re-run (or already-aligned DB) is a no-op.
    let alter_adds = migration.matches("ALTER TABLE").count();
    let not_exists_guards = migration
        .matches("NOT EXISTS (SELECT 1 FROM information_schema")
        .count();
    assert_eq!(alter_adds, 8, "expected 8 guarded column adds");
    assert!(
        not_exists_guards >= alter_adds,
        "each ALTER must sit behind a NOT EXISTS information_schema guard"
    );

    // Alias tables used by the search LEFT JOINs are ensured.
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS pornstar_aliases"));
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS channel_aliases"));

    // Idempotent + non-destructive: no table drops or truncations.
    assert!(!migration.to_uppercase().contains("DROP TABLE"));
    assert!(!migration.to_uppercase().contains("TRUNCATE"));

    // Backfills legacy columns rather than discarding data.
    assert!(migration.contains("UPDATE pornstars SET thumb_path = COALESCE(thumb_url"));
    assert!(migration.contains("UPDATE channels SET title = display_name"));
    assert!(migration.contains("UPDATE videos SET views = view_count"));
}
