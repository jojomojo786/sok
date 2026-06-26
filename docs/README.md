# PornsOK.com replica — analysis documentation

This folder captures how the **live site** (https://pornsok.com) behaves and how the **local Rust project** (`/Users/adeel/Downloads/sok`) compares. Goal: an **exact functional and visual replica** driven by real data (MySQL via `sqlx`), not static HTML dumps forever.

## How this was produced

- **Codebase**: read `src/`, `templates/`, `static/`, `Cargo.toml`, `config.json`.
- **Live site**: HTTP fetches of key URLs (titles, canonicals, H1s, approximate HTML size).
- **Templates**: parsed link patterns and section structure from saved PornsOK HTML in `templates/*.html`.
- **JavaScript**: reverse-engineered from `static/js/main.min.js` (jQuery + LazyLoad + AJAX widgets).

**Evidence refresh (2026-06-26):** Playwright capture under `docs/raw/live-inventory-2026-06-26/` (desktop + mobile). AJAX POST samples in `docs/raw/ajax-endpoint-samples.*`.

## Document index

| Doc | Purpose |
|-----|---------|
| [local-development.md](./local-development.md) | Run Actix locally, DB config, fixtures, `/health` and `/` checks (**sok-replica.10.3**) |
| [00-codebase-architecture.md](./00-codebase-architecture.md) | Actix + Askama stack, routes, static assets |
| [01-site-map-and-url-patterns.md](./01-site-map-and-url-patterns.md) | All page types and URL patterns |
| [02-replica-gap-analysis.md](./02-replica-gap-analysis.md) | What works today vs what's missing |
| [03-frontend-javascript-behavior.md](./03-frontend-javascript-behavior.md) | Client-side behavior and `/ajax/*` API |
| [template-partials.md](./template-partials.md) | Askama partial map + DOM hooks for mirrored templates |
| [catalog-schema.md](./catalog-schema.md) | MySQL catalog tables, slug rules, live DB discovery |
| [04-implementation-decisions.md](./04-implementation-decisions.md) | Durable build decisions (DB, routes, assets, player URLs) |
| [caching-hot-listing-queries.md](./caching-hot-listing-queries.md) | Hot listing / AJAX / search cache evaluation (`sok-replica.10.4`) |
| [05-template-partials-map.md](./05-template-partials-map.md) | Askama partial candidates + JS hook IDs |
| [pages/homepage.md](./pages/homepage.md) | `/` |
| [pages/categories.md](./pages/categories.md) | `/categories` |
| [pages/pornstars.md](./pages/pornstars.md) | `/pornstars` |
| [pages/channels.md](./pages/channels.md) | `/channels` |
| [pages/video-detail.md](./pages/video-detail.md) | `/video/{slug}.html` |
| [pages/category-tag-listing.md](./pages/category-tag-listing.md) | `/{slug}`, pagination |
| [pages/channel-profile.md](./pages/channel-profile.md) | `/channel/{slug}` |
| [pages/pornstar-profile.md](./pages/pornstar-profile.md) | `/pornstar/{slug}` |
| [pages/search.md](./pages/search.md) | Search + `/videos/{query}` |
| [pages/legal-static.md](./pages/legal-static.md) | `/page/*.html` |

## Beads

Epic: **sok-replica** — exact PornsOK replica roadmap; research evidence under **sok-replica.1**

## Suggested implementation order

1. Static file parity (`actix-files`) + catch-all routes for saved templates (short term).
2. Split monolithic HTML into Askama **partials** (header, footer, thumb macros).
3. Wire **MySQL models** for videos, categories, pornstars, channels.
4. Implement **`/ajax/*`** endpoints expected by `main.min.js`.
5. Video page: player container, related videos, comments, downloads.
6. Search (`/ajax/search_help`, listing pages).
7. Legal/static pages and SEO (canonical, pagination, `rel=next`).

## Raw evidence

See [docs/raw/README.md](./raw/README.md) for capture scripts and artifact layout.
