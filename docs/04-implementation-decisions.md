# Implementation decisions (durable)

Living record of choices made during the PornsOK replica build. Prefer this doc and Beads (`bd remember`, child issues) over chat history.

**Last updated:** 2026-06-26 (`sok-replica.10.4` caching evaluation)

## Database and configuration

| Topic | Decision |
|-------|----------|
| Primary DB | Aiven MySQL database `sok` from `DATABASE_URL` in `.env` (host/port/user/password in env, not committed). |
| `config.json` | `db.password` is **empty** in repo; runtime credentials come from `.env` / `DATABASE_URL` only. |
| Initial catalog | Live `sok` was **empty** (0 tables) on 2026-06-26. Authoritative DDL: `migrations/0001_catalog_schema.sql` (14 catalog tables). Taxonomy alignment: `migrations/001_taxonomy.sql`. Apply via `sqlx migrate` or direct SQL — see [catalog-schema.md](./catalog-schema.md). |
| Empty upstream | Do not invent production columns; if a populated upstream appears later, diff with `SHOW CREATE TABLE` and add `0002_*` migrations. |

## Routing and handlers (`src/handlers/mod.rs`)

| Topic | Decision |
|-------|----------|
| Route table | **sok-replica.2.1** complete plus **sok-replica.6.7** player paths: `/videofile/{token}`, `/embeded/{slug}.html`, `/video/{slug}.html`, profiles, `/page/{name}.html`, `/videos/{query}`, numeric `/{page}`, `/{slug}` fallback, `/ajax` stubs. Precedence covered by integration tests (`X-Sok-Handler` markers). |
| Dynamic HTML today | `GET /`, `/categories`, `/pornstars` render Askama templates with shared `RenderContext` + `PageMeta` (`src/views/context.rs`). |
| Stubs | Most other registered routes return **plain-text stubs** (`stub:…`) until dynamic templates and AJAX land — not production HTML yet. |
| AJAX | `GET /ajax` and `GET /ajax/{tail}` remain reserved placeholders. Video-page **POST** handlers for comments, related-video load-more, and vote UI are implemented in `src/handlers/ajax.rs`; other site-wide `/ajax/*` routes are covered by **sok-replica.5.1–5.4**. Persistent vote persistence is deferred (**sok-99v**). See [03-frontend-javascript-behavior.md](./03-frontend-javascript-behavior.md). |

## Static assets (`configure_static` in `src/lib.rs`)

| Mount | Purpose |
|-------|---------|
| `/static` | Full `static/` tree |
| `/fox-tpl` and `/static/fox-tpl` | Mirror production paths used in mirrored HTML/CSS |
| `/site.webmanifest` | Serves `static/site.webmanifest` at root URL templates expect |

**Local parity (sok-replica.7.1):** Mirrored under `static/fox-tpl/` with tiny placeholders where production art is non-critical: `images/loadMoreVideos.gif`, `images/spacer.gif`, `images/shadow.png`, `js/smiles_.json`, `js/playerjs.js`, KEmoji `style/rez/*/emoji.png`, `style/img/opacity.png`. Root icon URLs are served from `static/` via explicit routes in `configure_static` (`/favicon.ico`, `/apple-touch-icon.png`, `/favicon-32x32.png`, `/favicon-16x16.png`, `/safari-pinned-tab.svg`, manifest-linked `android-chrome-*.png`). **CDN-only (not vendored):** thumbs/posters/stream media on `https://c.foxporn.tv` (`fox-images/*`, `video/*`, category/pornstar menu `data-mini` URLs); inline CSS on some mirrors still references `https://c.foxporn.tv/fox-tpl/images/shadow.png` — local file exists for `/fox-tpl` and `/static/fox-tpl` mounts.

## Video playback URLs (live evidence + replica routes)

Live video detail (2026-06-26 inventory) uses:

| URL family | Example | Notes |
|------------|---------|-------|
| Player mount | `#player_container2` | Legacy `#player_container` not in live DOM; player JS targets `player_container2`. |
| Stream | `GET /videofile/{base64-token}` | `<video src="/videofile/…">` on live sample page. Handler: `video::videofile` — resolves migration `videos.stream_url` or dev `stream_token` when set and **302** to `preview_mp4_url` / `preview_mp4` (teaser MP4 placeholder until full CDN stream signing); unknown token **404** (not `/{slug}` fallback). |
| Player render context | `src/views/player_media.rs` | **sok-replica.6.2**: `VideoPlayerPageContext` with `PlayerMediaView` (poster, preview, `/videofile/{token}` or teaser MP4 fallback, optional `{cdn}/video/{slug}` download placeholder), `PlayerBootGlobals` (`isPLAYER = true`, `thumbs_path`, `video_path`, …). CDN host via `SiteLayout::with_media_cdn` / `SiteLayout::media_cdn` (default `https://c.foxporn.tv`). Wire into Askama in **sok-replica.6.1**. |

| Embed | `GET /embeded/{slug}.html` | Typo **embeded** preserved for parity; Schema.org `embedUrl`. Handler: `video::embeded_html` — minimal HTML5 `<video>` shell (placeholder) using `/videofile/{token}` when stream token/URL is present, else direct teaser MP4. Rust model: `embed_path()` in `src/models/video.rs`. |

Routes register in `src/handlers/mod.rs` **before** `/{slug}` category fallback. Covered by `tests/routes.rs` (`X-Sok-Handler` markers).


## Caching and hot reads (`sok-replica.10.4`)

| Topic | Decision |
|-------|----------|
| Runtime cache (`moka`) | **Deferred** — dependency present in `Cargo.toml` but not wired; no measured DB bottleneck yet. |
| Search AJAX (`search_help`, `search_cats_tags_queries`) | **No cache** until p95 latency is measured; high stale-risk and unbounded cache keys. |
| Homepage widgets (`update_*`) | Direct SQL on `week_views` / `weekly_views` / home listings; optional short TTL cache only after load tests. |
| Category counts | Index uses denormalized `video_count`; slug pagination uses live `COUNT` joins — optimize via maintained counts / indexes before app cache. |
| SQL precompute | Prefer `metric_snapshots` rollup job over query-string caching; see [caching-hot-listing-queries.md](./caching-hot-listing-queries.md). |
| Tests | Any future cache must default **off** in test `App` or be cleared per test to avoid cross-test stale JSON/HTML. |

## Taxonomy

Categories and tags use **separate tables** and slug resolution order documented in [schema/taxonomy.md](./schema/taxonomy.md) and `src/models/taxonomy.rs` — not a single shared slug namespace in SQL.

## Comments

Store `body_raw` + ammonia-sanitized `body_html`; **render only `body_html`** in templates/APIs (`sok-replica.3.7`).

## Evidence index

| Artifact | Location |
|----------|----------|
| Live screenshots + snippets | `docs/raw/live-inventory-2026-06-26/` + [manifest.json](./raw/live-inventory-2026-06-26/manifest.json) |
| AJAX POST samples | `docs/raw/ajax-endpoint-samples.{json,md}` |
| Fetch metadata summary | `docs/raw/live-fetch-summary.json` |
| Template partial map | [05-template-partials-map.md](./05-template-partials-map.md) |

## Beads epic map (implementation)

| Epic | Focus |
|------|--------|
| **sok-replica.1** | Research, evidence, docs sync |
| **sok-replica.2** | Actix routing, static, render context |
| **sok-replica.3** | MySQL catalog + models |
| **sok-replica.4** | Dynamic listing templates |
| **sok-replica.5** | AJAX + search |
| **sok-replica.6** | Video detail, player, comments |
| **sok-replica.7** | Frontend assets + visual parity |
| **sok-replica.8** | SEO, legal, favicons |
| **sok-replica.9** | Tests + browser QA |

Legacy doc reference `sok-la8` is superseded by the **sok-replica** tree — do not create new work under `sok-la8`.


## Video page AJAX (`sok-replica.5.6`)

`templates/video.html` + `static/js/main.min.js` call these local endpoints on `/video/{slug}.html`:

| POST endpoint | Handler | Replica behavior |
|---------------|---------|------------------|
| `/ajax/more_videos_3` | `more_videos_3` | JSON array batches for infinite-scroll related thumbs (`videourl` slug). |
| `/ajax/add_hit/more_videos` | `add_hit_more_videos` | Benign `ok` analytics ack when user clicks **Show more**. |
| `/ajax/more_comments` | `post_more_comments` | JSON `{ comments: "<html>" }` for paged comment boxes. |
| `/ajax/comments` | `post_comments` | JSON `{ result, text }` for KEmoji comment submit (`vid` video id). |
| `/ajax/add_vote_v3` | `add_vote_v3` | Safe local stub JSON `{ raiting, msg }`; returns current like % without persisting votes (**sok-99v**). |

Header bookmark still uses `/ajax/add_hit/favourite` (site-wide, already implemented).
