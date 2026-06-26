# Catalog MySQL schema (PornsOK replica)

Reference for Rust `sqlx` models and queries. Authoritative DDL: [`migrations/0001_catalog_schema.sql`](../migrations/0001_catalog_schema.sql) and [`migrations/001_taxonomy.sql`](../migrations/001_taxonomy.sql).

> **Entity/video column alignment (sok-replica.3.8):** the `sqlx` models query
> `pornstars.thumb_path`, `channels.title`, `*.week_views`, and the `videos`
> columns `views` / `status` / `wide_thumb`. The canonical entity/video column
> set lives in `ENTITY_SCHEMA_SQL` (`src/models/entities.rs`) and
> [`sql/schema/videos.sql`](../sql/schema/videos.sql). `0001` predates that and
> uses `display_name` / `thumb_url` / `view_count`; a catalog provisioned from
> `0001` (e.g. the live Aiven DB) must apply the additive
> [`migrations/0002_align_catalog_search_thumbs.sql`](../migrations/0002_align_catalog_search_thumbs.sql)
> migration so header autocomplete (`/ajax/search_help`) and in-page entity
> search query the live DB instead of falling back to fixtures.

## Live database discovery

| Check | Result (2026-06-26) |
|-------|---------------------|
| `DATABASE_URL` in `.env` | Present (Aiven host `mysql-1158a03d-wahababdul-bea8.c.aivencloud.com:22451`, database `sok`) |
| TCP/auth | OK via `pymysql` |
| `SHOW TABLES` | **0 tables** (empty catalog) |
| `mysql` CLI | Not installed on dev host |
| `config.json` `db.password` | Empty (credentials only in `.env`) |

**Conclusion:** No production schema to introspect. Apply `0001_catalog_schema.sql` as the initial catalog (sqlx migrate or manual apply). If a populated upstream schema appears later, diff with `SHOW CREATE TABLE` and add `0002_*` alignment migrations rather than editing `0001` in place.

## Slug rules

Slugs are **unique per entity family** (separate `UNIQUE` constraints, not one global namespace):

| Table | URL pattern | Unique key |
|-------|-------------|------------|
| `videos` | `/video/{slug}.html` | `uq_videos_slug` |
| `categories` | `/{slug}` | `uq_categories_slug` |
| `tags` | `/{slug}` or tag hub | `uq_tags_slug` |
| `channels` | `/channel/{slug}` | `uq_channels_slug` |
| `pornstars` | `/pornstar/{slug}` | `uq_pornstars_slug` |
| `pages` | `/page/{slug}.html` | `uq_pages_slug` + `uq_pages_path` |

**Routing note:** App layer must reserve paths (`/categories`, `/pornstars`, `/channels`, `/video`, `/page`, numeric home pagination `/2`…) so category/tag slugs do not collide with static routes.

`VARCHAR(191)` slugs support utf8mb4 unique indexes under InnoDB’s 3072-byte limit.

## Entity map

```text
categories ──< video_categories >── videos
tags         ──< video_tags         >── videos
pornstars    ──< video_pornstars    >── videos
channels     ──< video_channels     >── videos
channels     ──< videos.primary_channel_id (optional)

videos ──< comments
videos ──< video_downloads
videos ── media_assets (entity_type=video)

metric_snapshots — weekly/trending scores per entity_type
pages — legal/static HTML bodies
```

## Column ↔ UI mapping

### Video thumb (home / listings)

From `templates/index.html` `.thumb.vid` markup:

| UI / schema.org | Column(s) |
|-----------------|-----------|
| `href="/video/{slug}.html"` | `videos.slug` |
| `thumb-title` | `videos.title` |
| `ttime` / `PT###S` | `videos.duration_seconds` |
| `tview` / WatchAction count | `videos.view_count` |
| `tlike` % | `videos.like_percent` |
| CommentAction count | `videos.comment_count` |
| `datePublished` | `videos.published_at` |
| `data-original` poster | `videos.thumb_url` |
| `data-video` preview | `videos.preview_mp4_url` |
| `not-wide` / `widethumb` (search) | `videos.is_wide_thumb`, `wide_thumb_url` |
| `?sort=mv` / `mc` / `?hd=1` | indexes on `view_count`, `comment_count`, `is_hd` |

CDN convention (mirror): `https://c.foxporn.tv/fox-images/videos/{slug}.jpg` and `m-{slug}.mp4`.

### Taxonomy & profiles

| Page doc | Primary tables |
|----------|----------------|
| `docs/pages/category-tag-listing.md` | `categories`, `video_categories`, optional `tags` |
| `docs/pages/channels.md` | `channels` |
| `docs/pages/channel-profile.md` | `channels`, `videos`, `video_channels` |
| `docs/pages/pornstar-profile.md` | `pornstars`, `video_pornstars` |
| `docs/pages/search.md` | `videos`, `pornstars`, `channels` (unified search is app/query layer) |
| `docs/pages/video-detail.md` | `videos`, joins, `comments`, `video_downloads`, `media_assets` |
| `docs/pages/legal-static.md` | `pages` (`slug`, `path`, `body_html`) |

### Comments & metrics

- **Comments:** store `body_raw` + ammonia-sanitized `body_html`; `videos.comment_count` maintained by ingestion or triggers (app responsibility).
  - **`body_raw`:** trimmed text; KEmoji editor HTML is normalized to `$#emoji_name#$` tokens before insert (`sok::models::comments`).
  - **`body_html`:** produced only by `prepare_comment_body` / `sanitize_comment_html` (ammonia + allowed KEmoji `<i>` markup). **Render `body_html` only** in templates/APIs.
- **Metrics:** `metric_snapshots` backs AJAX widgets (`update_pornstars`, `update_channels`, `update_tags`) with `period_*` and `score`.

## Suggested sqlx query surfaces

| Feature | Tables / filters |
|---------|------------------|
| Home feed page N | `videos` WHERE `is_active` ORDER BY `published_at DESC` LIMIT/OFFSET |
| Category listing | `videos` JOIN `video_categories` WHERE `categories.slug = ?` + sort/hd |
| Video detail | `videos` by slug + tag/category/channel/star joins |
| Related videos | shared `video_tags` / `primary_channel_id` |
| AJAX weekly blocks | `metric_snapshots` or `is_featured` + `video_count` on stars/channels/tags |
| Legal page | `pages` WHERE `path = ?` AND `is_published` |

## Apply migration

```bash
# After installing sqlx-cli (optional):
export DATABASE_URL='mysql://...'
sqlx migrate run

# Or apply SQL directly (verified in dev):
python3 -c "..."  # see issue notes / pymysql execute script
```

## Follow-ups (other beads)

- `sok-replica.3.3` — Rust taxonomy models
- `sok-replica.3.4` — channel/pornstar models
- `sok-replica.3.6` — seed from mirrored templates
- Wire `sqlx::migrate!()` in app startup when team enables auto-migrate
