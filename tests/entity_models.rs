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

/// Splitter-compatibility regression for sok-replica.3.8.
///
/// `src/fixtures/mod.rs::execute_sql_script` runs migrations with a naive
/// `script.split(';')` loop that trims each chunk and skips any chunk whose
/// trimmed text `starts_with("--")`. An earlier revision of the 0002 migration
/// placed `-- section` comments immediately before each executable statement,
/// so after splitting on `;` those chunks began with `--` and were dropped
/// wholesale -- leaving `@ddl` unset/stale when `PREPARE stmt FROM @ddl` ran.
///
/// This test replicates that exact splitter and proves every intended
/// executable statement survives it (and that no comment prose leaks through
/// as a statement the runner would try to execute).
#[test]
fn alignment_migration_survives_execute_sql_script_splitter() {
    let migration = include_str!("../migrations/0002_align_catalog_search_thumbs.sql");

    // Exact mirror of execute_sql_script's chunk selection.
    let executed: Vec<&str> = migration
        .split(';')
        .map(str::trim)
        .filter(|stmt| !stmt.is_empty() && !stmt.starts_with("--"))
        .collect();

    // 1. Nothing the runner would skip (a `--`-leading chunk) may still contain
    //    executable SQL -- that is the precise defect being guarded against.
    let dropped_with_sql = migration
        .split(';')
        .map(str::trim)
        .filter(|stmt| stmt.starts_with("--"))
        .filter(|stmt| {
            stmt.lines()
                .map(str::trim)
                .any(|line| !line.is_empty() && !line.starts_with("--"))
        })
        .count();
    assert_eq!(
        dropped_with_sql, 0,
        "a comment-leading chunk hides executable SQL the splitter will drop"
    );

    // 2. Every executed chunk must actually start with a SQL keyword, so no
    //    comment prose (e.g. a `;` inside a `--` line) leaks through as a query.
    let sql_starts = [
        "SET ",
        "PREPARE ",
        "EXECUTE ",
        "DEALLOCATE ",
        "CREATE TABLE",
        "ALTER ",
        "UPDATE ",
        "INSERT ",
    ];
    for stmt in &executed {
        let first = stmt.lines().next().unwrap_or("").trim();
        assert!(
            sql_starts.iter().any(|kw| first.starts_with(kw)),
            "non-SQL chunk would be executed by the splitter: {first:?}"
        );
    }

    // 3. The guarded dynamic-DDL trio must all survive intact and balanced, and
    //    both alias tables must be created. (14 guards = 8 column adds + 6
    //    backfills.)
    let count = |prefix: &str| executed.iter().filter(|s| s.starts_with(prefix)).count();
    assert_eq!(
        count("SET @ddl"),
        14,
        "guarded dynamic statements were dropped"
    );
    assert_eq!(count("PREPARE"), 14, "PREPARE statements were dropped");
    assert_eq!(count("EXECUTE"), 14, "EXECUTE statements were dropped");
    assert_eq!(
        count("DEALLOCATE"),
        14,
        "DEALLOCATE statements were dropped"
    );
    assert_eq!(
        count("CREATE TABLE"),
        2,
        "alias table DDL was dropped by the splitter"
    );

    // Every PREPARE must be paired with a non-empty @ddl assignment before it,
    // i.e. there are exactly as many guards as PREPAREs.
    assert_eq!(
        count("SET @ddl"),
        count("PREPARE"),
        "each PREPARE must be preceded by a surviving SET @ddl guard"
    );
}
