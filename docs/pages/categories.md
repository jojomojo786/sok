# Categories index (`/categories`)

## Live reference

- **URL:** https://pornsok.com/categories
- **Canonical:** `https://pornsok.com/categories`
- **Title:** Porn Categories: Hottest Sex Niches | PornsOK.com
- **H1:** Porn Video Categories
- **Local mirror:** `templates/categories.html` (~169 KB)
- **Local route:** `GET /categories` → `CategoriesTemplate`

## Intro copy

`.toptext-container` > `.toptext` — SEO paragraph with bold "porn categories"; clamp/expand via `.arrow-clamp` JS on long text pages.

## In-page search

| ID | Purpose |
|----|---------|
| `#search-genres-input` | Filter categories/tags |
| `#search-genres-x` | Clear |
| `#ajax_content` | AJAX results container |

POST `/ajax/search_cats_tags_queries` returns JSON `{ search_text, items: [{ name, url }] }` rendered as `.tags-list` ul with `.fa-tag`.

During search: hide `.all_cats`, change H1 to `Searching "{query}"...`.

## Main grid (`.all_cats`)

- Items: `.thumb.cat` (not `.vid`).
- Thumb image from `https://c.foxporn.tv/fox-images/categories/{slug}.jpg`.
- `.count-videos` badge with camera SVG + integer count.
- Title centered `.thumb-title`.
- Example slugs in mirror: `/lesbian`, `/milf`, `/anal`, … (~154 category links).

## Secondary sections

1. **Today's Top Viewed Tags** — H2 with `.fa-tags`.
2. **Today's Top Viewed Pornstars And Models** — H2 with `.fa-star`; links to `/pornstar/...`.

## Shared chrome

Same header/footer/theme as homepage (inline CSS block duplicated across templates).

## Filters

Live capture (2026-06-26): **no** `.filter-section` on `/categories`; sorting/filter UI is limited to the in-page category search (`#search-genres-input`) and secondary “top viewed” blocks.

## href breakdown (local mirror)

| Type | Count |
|------|-------|
| category_slug | 154 |
| pornstar | 29 |
| channel | 17 |
| legal/nav | 11 |

## Backend requirements

- Table(s): categories with `slug`, `title`, `thumb_url`, `video_count`.
- Tags may be separate or unified search index.
- Endpoint: `/ajax/search_cats_tags_queries`.
- Optional: top viewed tags/pornstars queries (weekly/daily).

## Gap

Route exists but content is static; AJAX and search will fail against local server until implemented.

## Live inventory evidence (2026-06-26)

- Captured with headless Chromium (Playwright): desktop 1440×900, mobile 390×844.
- Manifest: `docs/raw/live-inventory-2026-06-26/manifest.json`
- Screenshots/snippets: `docs/raw/live-inventory-2026-06-26/categories__desktop.*`, `docs/raw/live-inventory-2026-06-26/categories__mobile.*`
