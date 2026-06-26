# Mirrored template → Askama partial map

Supports **sok-replica.1.2** (decomposition before dynamic data). Sources: `templates/index.html`, `templates/categories.html`, `templates/pornstars.html` (~125–202 KB each).

**Rule:** IDs and classes referenced in `static/js/main.min.js` must survive extraction unchanged.

## Shared blocks (high duplication)

| Partial candidate | DOM / markers | Present on | JS / behavior hooks |
|-------------------|---------------|------------|---------------------|
| `partials/head_meta.html` | `<head>` through `</style>` (theme CSS variables, `@font-face`, base layout CSS) | All three | OG/RTA/canonical via `RenderContext`; font URLs `/fox-tpl/fonts/custom4/*` |
| `partials/svg_sprites.html` | Hidden SVG defs (`#film-svg`, `#camera-svg`, `#player-svg`, `#hd-svg`, …) | All three | `<use xlink:href="#…">` in thumbs and UI |
| `partials/header.html` | `.header`, `.header-in`, `.logotype`, `.header-menu`, `.nav` / `.nav-in`, mega menu `.menu-pic`, `.sub-url`, `.head-url` | All three | `#main-search`, `#search_result`, `#day-night`, `.btn-search`, `.search-box` |
| `partials/footer.html` | `.footer`, `.footer-menu`, `.footer-copyr`, responsive `picture` logo | All three | Legal links `/page/*.html` |
| `partials/msg_toast.html` | `#msg_container`, `#text_msg` | All three | `show_msg()` |
| `partials/scripts_boot.html` | Inline globals + deferred `main.min.js` | All three | `directory`, `thumbs_path`, `isTHUMBS_OR_PLAYER`, `isPLAYER`, `lazyThreshold` |
| `partials/mobile_shell.html` | `#side-panel`, `#close-overlay`, `#gotop` | Injected by JS on `document.ready` today | `.btn-mob`, `.head-url`/`sub-url` clone — optional to pre-render in partial for no-FOUC |

Head `<style>` blocks differ slightly per page (hash differs) but share one design system — consolidate to a single `site.css` or one partial with identical `:root` / `[data-theme="dark"]` rules.

## Thumb macros

| Macro | Markup root | Used on | Selectors to preserve |
|-------|-------------|---------|------------------------|
| `thumb_video.html` | `.thumb.vid` | Home grid, watching now AJAX, listings | `.thumb-cover`, `data-original`, `data-video`, `.video-preview`, `.ttime`, `.tview`, `.tlike`, `.thumb-title`, Schema.org counters |
| `thumb_cat.html` | `.thumb.cat` | Categories grid, pornstars grid, AJAX `update_pornstars` / `update_channels` | `.count-videos`, centered `.thumb-title`, category JPG CDN path |
| `thumb_search_row.html` | `#search_result li` classes `pornstars` / `channels` / `videos` | Header AJAX | Built in JS via `createSearchItem()` — server JSON must match field names in [raw/ajax-endpoint-samples.md](./raw/ajax-endpoint-samples.md) |

## Page-specific main content

| Page | Container IDs | Page-only partial |
|------|---------------|-------------------|
| Home `/` | `#ajax_pornstars`, `#ajax_channels`, `#ajax_tags`, `#ajax_watching_now`, `.thumbs-floats`, `.page_nav`, `.filter-section`, `.desc-text` | `pages/home_main.html` |
| Categories | `#search-genres-input`, `#search-genres-x`, `#ajax_content`, `.all_cats`, `.toptext-container` | `pages/categories_main.html` — **no** `.filter-section` on live `/categories` (2026-06-26) |
| Pornstars | `#search-page-input`, `#search-page-x`, `#search-box-page`, `.all_pornstars`, `.page_nav`, `.filter-section` | `pages/pornstars_main.html` — embed `search_type` global for `/ajax/search_{type}` |

## Pagination + filters

| Partial | Markers | Notes |
|---------|---------|-------|
| `partials/pagination.html` | `.page_nav`, `.pagination` | Home uses numeric `/2`…; pornstars index uses `/pornstars/2` pattern in `PageMeta.rel_next` |
| `partials/filter_sort.html` | `.filter-section`, `.sort-hd` | Home + pornstars; omit on categories index per live capture |

## Decomposition order (suggested)

1. `head_meta` + `svg_sprites` + `header` + `footer` + `scripts_boot` (unlock all listing pages).
2. `thumb_video` / `thumb_cat` macros (unlock AJAX HTML fragment renderers).
3. Page mains per route family.
4. Save a production video HTML shell → `templates/video.html` with `#player_container2` (**sok-replica.6.1**).

## Cross-links

- [00-codebase-architecture.md](./00-codebase-architecture.md) — Askama structs today
- [03-frontend-javascript-behavior.md](./03-frontend-javascript-behavior.md) — AJAX DOM targets
- [02-replica-gap-analysis.md](./02-replica-gap-analysis.md) — what is still static vs stub routes
