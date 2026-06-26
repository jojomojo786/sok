# Caching strategy for hot listing queries

**Last updated:** 2026-06-26  
**Status:** Documented evaluation — **no in-process or SQL result cache wired in the app yet.**

| Decision | Choice |
|----------|--------|
| Add `moka` in handlers now? | **No** — no measured production bottleneck; empty/small dev DB; high stale-data risk for search and counts. |
| Add MySQL query-result caching now? | **No** — prefer denormalized columns + indexes already in schema; use `metric_snapshots` only after a batch job exists. |
| What to do instead? | Keep direct `sqlx` queries; rely on fixture fallbacks for tests/dev; add **observability** (slow-query logging / `EXPLAIN`) before any cache layer. |
| When to revisit? | After catalog seed at realistic scale, p95 handler+DB latency targets missed, or weekly widget traffic dominates DB CPU. |

This matches the bead design note: *add caching only after dynamic queries exist and bottlenecks are measurable.* Dynamic listing and AJAX paths are implemented; bottlenecks are **not** measured yet.

## Hot surfaces in scope

### Homepage AJAX widgets (`POST /ajax/update_*`)

| Endpoint | Loader | Primary SQL | Result size | Volatility |
|----------|--------|-------------|-------------|------------|
| `update_pornstars` | `load_widget_pornstars` → `list_top_pornstars_week` | `pornstars` ORDER BY `week_views`, `video_count` LIMIT 30 | Small | Medium (weekly metrics) |
| `update_channels` | `load_widget_channels` → `list_top_channels_week` | `channels` ORDER BY `week_views`, `video_count` LIMIT 30 | Small | Medium |
| `update_tags` | `load_widget_tags` → `list_top_viewed_tags` | `tags` WHERE `is_active` ORDER BY `weekly_views` LIMIT 24 | Small | Medium |
| `update_watching_now` | `load_widget_watching_now` → `list_watching_now_thumbs` | `videos` ORDER BY `views`, `uploaded_at` LIMIT 84 | Large HTML | High |
| `update_newest_videos` | `load_widget_newest_videos` → `list_home_thumbs` (newest) | Paginated home feed LIMIT 12–48 | Medium | High |

Widgets use **DB-first, fixture fallback** when the pool errors or returns empty (`src/handlers/ajax.rs`). Refresh buttons mean users can hit the same endpoint repeatedly in one session; any cache must tolerate **rotation** semantics (live site may return a different slice; replica currently returns deterministic ORDER BY until `metric_snapshots` rotation is implemented).

### Category counts and index blocks

| Surface | Query | Notes |
|---------|-------|-------|
| `/categories` grid | `list_categories_for_index` | Uses denormalized `categories.video_count` — **no live `COUNT(*)` over join** for the index grid. |
| `/categories` top tags | `list_top_viewed_tags` | Uses `tags.weekly_views` + `video_count` (indexed: `idx_tags_active_weekly`). |
| Slug listing pagination | `count_videos_for_category` / `count_videos_for_tag` | **Heavy**: `COUNT(DISTINCT v.id)` with join + optional `hd` filter — prime future SQL optimization target (maintained counts or covering indexes), not app-level cache first. |
| Entity index totals | `count_pornstars` / channel equivalents | Simple `COUNT(*)` on single table — cheap at replica scale. |

### Search suggestions

| Endpoint | Function | Pattern | Staleness risk |
|----------|----------|---------|----------------|
| `POST /ajax/search_help` | `search_help_from_db` | Empty query → top 6 per group by popularity; typed query → `LIKE` on names/aliases/titles | **High** if TTL > few seconds — autocomplete must reflect new titles/aliases quickly. |
| `POST /ajax/search_cats_tags_queries` | `search_categories_and_tags` | `UNION` of categories + tags with alias subqueries, `LIKE`, LIMIT 80 | **High** — keyspace is `(text, limit)` with huge cardinality. |
| `POST /ajax/search_{type}` | `search_entities_for_page` | Entity grid search LIMIT 120 | Medium |
| `GET /videos/{query}` | `list_search_videos` + `count_search_videos` | Title `LIKE` + sort/hd + pagination | Medium — full page, not autocomplete |

All search AJAX handlers mirror widgets: **DB first**, then bundled `fixtures/catalog_seed.json` when DB is empty or errors (`tests/ajax_search_help.rs`, `tests/ajax_cats_tags.rs` use this path when MySQL has no rows).

## `moka` evaluation (in-process)

`moka` is listed in `Cargo.toml` (`features = ["sync"]`) but **not referenced** under `src/`. Pros and cons for this codebase:

**Pros**

- Cheap win for **identical** repeated reads (e.g. `update_pornstars` HTML for the same limit within a short TTL).
- No extra infrastructure; fits Actix `web::Data` shared cache.

**Cons (why defer)**

1. **No baseline latency data** — Dev catalog was empty on 2026-06-26; integration tests often exercise fixture fallbacks, not hot MySQL paths.
2. **Test and dev surprise** — Integration tests build a real pool per test (`tests/ajax_widgets.rs`, `tests/search_results.rs`). A process-global `moka` cache would serve **stale HTML/JSON across tests** unless every test uses `moka::sync::Cache::invalidate_all()` or caches are injected and disabled in test `App` wiring. That violates acceptance: *do not serve stale data in tests unexpectedly.*
3. **Search cardinality** — `search_help` and `search_cats_tags_queries` keys are unbounded strings; an LRU must be tightly capped and still risks wrong suggestions after ingest.
4. **Fixture contract** — Empty DB must keep returning seed-shaped responses; caching a “DB returned empty once” sentinel would mask migrations applied mid-process.

**If added later (recommended shape)**

- `CacheLayer` in `src/cache.rs` with **explicit opt-in** per query family, not global middleware.
- Separate caches: `widget_snapshot` (TTL 60–300s, keys like `pornstars:30`), **no cache** on typed search strings until metrics justify it.
- `cfg(test)` or `Config::cache_enabled: false` default in test profile; document in test harness.
- Invalidate on write paths (comment submit, future admin ingest) — none of the hot read paths have matching hooks today.

## SQL-side caching / precomputation

Prefer **schema-aligned precomputation** over caching raw query strings:

- `categories.video_count`, `tags.video_count`, `tags.weekly_views`
- `pornstars.week_views`, `channels.week_views`, `video_count` on entities

`metric_snapshots` (`migrations/0001_catalog_schema.sql`) is the right place for **precomputed weekly rankings** once a job rolls up views into `score` / `period_end`. Widget loaders should read snapshots **instead of** sorting live fact tables at scale — that is **materialized data**, not MySQL `QUERY CACHE` (removed in MySQL 8) or ad hoc `CACHE TABLE`.

| Approach | Fit | Action now |
|----------|-----|------------|
| Denormalized counts on taxonomy/entities | Already used for index cards | Keep ingest/job responsible for refreshing counts when videos link/unlink |
| `metric_snapshots` for `update_*` rotation | Schema ready; loaders use `week_views` columns | Add batch rollup bead before switching widget SQL |
| Maintained `category_id → video_count` for HD/all | Not present | Consider trigger or nightly job **before** caching `count_videos_for_category` results |
| Read replica | Aiven supports replicas | Operational scaling later; does not fix stale autocomplete |

## Invalidation and parity risks

1. **Video publish/unpublish** — Home, watching-now, search video group, counts.
2. **Search AJAX:** TTL ≤ 30s or no cache; empty-query “popular” suggestions may use longer TTL (60–120s) **only** with invalidation on entity/title updates.
3. **Widgets:** Accept 1–5 minute staleness for weekly rails **only** if product matches live site (live refresh may expect new rotation; document parity).
4. **Counts on slug listings:** Prefer **correct** maintained counts over cached `COUNT(*)`; stale pagination totals are worse than slow queries.

## Measurement checklist (before implementation)

1. Seed MySQL with `fixtures/catalog_seed.json` / `DEV_CATALOG_SQL` at minimum; ideally production-sized import.
2. Log `sqlx` slow statements (>50ms) for: `search_help_from_db`, `search_categories_and_tags`, `count_videos_for_category`, `list_watching_now_thumbs`.
3. Run `EXPLAIN ANALYZE` on those statements with `hd=1` variants.
4. Load-test `POST /ajax/update_watching_now` and `POST /ajax/search_help?text=a` at concurrent levels (e.g. 50 RPS) and record p95 latency.
5. Only if p95 exceeds agreed SLO (e.g. 200ms DB portion), implement the **smallest** fix: index/maintained count → `metric_snapshots` → `moka` widget TTL.

## Related code and docs

- Handlers: `src/handlers/ajax.rs`, `src/handlers/listings.rs`, `src/views/categories_data.rs`
- Models: `src/models/search_help.rs`, `src/models/taxonomy.rs`, `src/models/entities.rs`, `src/models/video.rs`
- Schema: `docs/catalog-schema.md`, `migrations/0001_catalog_schema.sql`, `migrations/001_taxonomy.sql`
- Architecture note: `docs/00-codebase-architecture.md` (moka dependency)
