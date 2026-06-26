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

/// Regression guard for sok-replica.3.9: the live Aiven taxonomy was created
/// before `001_taxonomy.sql` grew `is_active` / `intro_html` / `weekly_views`,
/// and `001_taxonomy.sql` only uses `CREATE TABLE IF NOT EXISTS`, so it never
/// altered the legacy `categories` / `tags` tables. POST
/// `/ajax/search_cats_tags_queries` then failed with `Unknown column
/// 'is_active' in 'where clause'`. The additive
/// `0003_align_taxonomy_search_schema.sql` migration must add every taxonomy
/// column the search query depends on, non-destructively and idempotently.
#[test]
fn taxonomy_alignment_migration_adds_search_columns() {
    let migration = include_str!("../migrations/0003_align_taxonomy_search_schema.sql");

    for token in [
        "ALTER TABLE categories ADD COLUMN intro_html",
        "ALTER TABLE categories ADD COLUMN is_active",
        "ALTER TABLE tags ADD COLUMN weekly_views",
        "ALTER TABLE tags ADD COLUMN is_active",
    ] {
        assert!(
            migration.contains(token),
            "0003 migration missing column alignment: {token}"
        );
    }

    // Each column add sits behind an information_schema NOT EXISTS guard so a
    // re-run (or already-aligned DB) is a no-op.
    let alter_adds = migration.matches("ALTER TABLE").count();
    let not_exists_guards = migration
        .matches("NOT EXISTS (SELECT 1 FROM information_schema")
        .count();
    assert_eq!(alter_adds, 4, "expected 4 guarded taxonomy column adds");
    assert!(
        not_exists_guards >= alter_adds,
        "each ALTER must sit behind a NOT EXISTS information_schema guard"
    );

    // The alias table the search EXISTS-join reads is ensured.
    assert!(migration.contains("CREATE TABLE IF NOT EXISTS taxonomy_search_aliases"));

    // Idempotent + non-destructive: no drops or truncations.
    let upper = migration.to_uppercase();
    assert!(!upper.contains("DROP TABLE"));
    assert!(!upper.contains("DROP COLUMN"));
    assert!(!upper.contains("TRUNCATE"));

    // Dynamic SQL must not use double-quoted string literals, which break under
    // ANSI_QUOTES sessions where "..." is parsed as an identifier. All literals
    // are single-quoted with doubled single quotes for escaping.
    assert!(
        !migration.contains("ADD COLUMN intro_html TEXT NULL\""),
        "dynamic DDL must use single-quoted literals"
    );
    for line in migration.lines() {
        let trimmed = line.trim();
        // Block-comment prose and the SET NAMES header may legitimately contain
        // quotes; only the executable PREPARE'd literals must avoid `"`.
        if trimmed.starts_with("'ALTER TABLE") || trimmed.starts_with("'UPDATE") {
            assert!(
                !trimmed.contains('"'),
                "double-quoted literal in dynamic SQL would break under ANSI_QUOTES: {trimmed}"
            );
        }
    }
}

/// Splitter-compatibility regression for sok-replica.3.9.
///
/// `src/fixtures/mod.rs::execute_sql_script` runs migrations with a naive
/// `script.split(';')` loop that trims each chunk and skips any chunk whose
/// trimmed text `starts_with("--")`. The 0003 migration must survive that
/// splitter: every intended executable statement is preserved, and no comment
/// prose leaks through as a statement the runner would try to execute.
#[test]
fn taxonomy_alignment_migration_survives_execute_sql_script_splitter() {
    let migration = include_str!("../migrations/0003_align_taxonomy_search_schema.sql");

    let executed: Vec<&str> = migration
        .split(';')
        .map(str::trim)
        .filter(|stmt| !stmt.is_empty() && !stmt.starts_with("--"))
        .collect();

    // No `--`-leading chunk may hide executable SQL the splitter would drop.
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

    // Every executed chunk must start with a SQL keyword, so no comment prose
    // leaks through as a query.
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

    // The guarded dynamic-DDL quartet must all survive intact and balanced, and
    // the alias table must be created. (4 guards = 4 column adds.)
    let count = |prefix: &str| executed.iter().filter(|s| s.starts_with(prefix)).count();
    assert_eq!(
        count("SET @ddl"),
        4,
        "guarded dynamic statements were dropped"
    );
    assert_eq!(count("PREPARE"), 4, "PREPARE statements were dropped");
    assert_eq!(count("EXECUTE"), 4, "EXECUTE statements were dropped");
    assert_eq!(count("DEALLOCATE"), 4, "DEALLOCATE statements were dropped");
    assert_eq!(
        count("CREATE TABLE"),
        1,
        "alias table DDL was dropped by the splitter"
    );
    assert_eq!(
        count("SET @ddl"),
        count("PREPARE"),
        "each PREPARE must be preceded by a surviving SET @ddl guard"
    );
}
