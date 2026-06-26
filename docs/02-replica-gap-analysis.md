# Replica gap analysis

## Summary

| Area | Live PornsOK | Local `sok` | Gap severity |
|------|--------------|-------------|--------------|
| Home `/` | Dynamic + AJAX | Static mirror, route OK | High (data + ajax) |
| Pagination `/N` | Yes | No routes | High |
| Categories index | Yes | Static mirror, route OK | Medium |
| Pornstars index | Yes | Route + static mirror | Medium (data + ajax) |
| Channels index | Yes | Route stub only | High |
| Slug listings `/milf` | Yes | Route stub only | High |
| Video pages | Yes | Route stub only | Critical |
| Search | Yes | Route stub only | High |
| `/ajax/*` | Yes | Nothing | High |
| MySQL content | Full catalog | Empty DB + migrations applied in dev | Critical |
| Askama variables | N/A | `RenderContext` on 3 listings | Medium |
| Theme toggle | JS + CSS vars | HTML present, needs JS + cookie | Low |
| CDN thumbs | c.foxporn.tv | Hard-coded in HTML | Medium (configurable base URL) |

## What already matches well

- **Visual shell** for home, categories, pornstars (saved from production).
- **CSS design system**: `[data-theme="dark"]` with extensive `--color-*` and `--gradient-*` variables inlined in templates.
- **Icon font** and SVG sprites (`#film-svg`, `#camera-svg`, etc.).
- **Client bundle** `static/js/main.min.js` aligned with production behavior expectations.
- **Dependencies** in `Cargo.toml` match a production-ready stack (cache, ammonia for sanitization, chrono, urlencoding).

## Code-level gaps

### Routing (`src/handlers/mod.rs`)

**Done (sok-replica.2.1):** GET route table + precedence tests for all URL families above (most return stubs).

**Still missing:**

- `POST /ajax/*` handlers matching [raw/ajax-endpoint-samples.md](./raw/ajax-endpoint-samples.md)
- `GET /videofile/{token}` and `GET /embeded/{slug}.html` for player parity (live evidence; see [04-implementation-decisions.md](./04-implementation-decisions.md))
- Root favicon / PWA icon files and routes (`/favicon.ico`, etc.)
- Dynamic Askama output + SQL for stub routes

### Views (`src/views/mod.rs`)

**Done:** `RenderContext`, `PageMeta`, `SiteLayout`, `AssetPaths` on home/categories/pornstars.

**Still needed on listing/video templates:**

- `videos: Vec<VideoThumb>`
- `pagination: Pagination`
- `sort_options`, `active_sort`, `hd_only`

### Models (`src/models/mod.rs`)

In progress: `video`, `taxonomy`, `pagination`, `comments` modules aligned to `migrations/0001_catalog_schema.sql` and `001_taxonomy.sql`. Seed/fixtures: **sok-replica.3.6**.

### Templates

Monolithic HTML; must decompose without changing DOM hooks used by jQuery.

### Security / content

- `ammonia` dependency suggests HTML sanitization for comments — not implemented.
- RTA meta tag present in templates.

## Live vs local template size parity

| Page | Live HTML len (fetch) | Local template bytes |
|------|----------------------|----------------------|
| `/` | ~206191 | ~201880 |
| `/categories` | ~170741 | ~169004 |
| `/pornstars` | ~126152 | ~124514 |

Sizes are close — mirrors are recent.

## Chrome / visual QA (recommended next step)

When Chrome plugin is available:

1. Side-by-side localhost:8080 vs pornsok.com for header, thumb hover preview, dark mode.
2. Capture breakpoints (949px footer logo swap is in template).
3. Video page player controls and related rail.

Document screenshots under `docs/raw/` (folder created for dumps).

## Template decomposition

See [05-template-partials-map.md](./05-template-partials-map.md) (bead **sok-replica.1.2**).

## Implementation beads

Work is tracked under epic **sok-replica** (not legacy `sok-la8`): **sok-replica.4** (dynamic pages), **sok-replica.5** (AJAX), **sok-replica.6** (video), **sok-replica.3** (catalog).
