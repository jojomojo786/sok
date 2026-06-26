# Pornstars index (`/pornstars`)

## Live reference

- **URL:** https://pornsok.com/pornstars
- **Canonical:** `https://pornsok.com/pornstars`
- **Title:** Best Pornstars and Models in Free Porn Videos | PornsOK.com
- **H1:** Top Trending Pornstars
- **Local mirror:** `templates/pornstars.html` (~125 KB)
- **Local route:** `GET /pornstars` → `PornstarsTemplate` (mirrored HTML; in-page AJAX still requires POST `/ajax/search_{type}`)

## Layout

- `.sect-title-wrap` with H1 and star SVG sprite `#star-svg`.
- `#search-box-page` with `#search-page-input`, `#search-page-x` — placeholder for filtering models.
- Grid container: `.all_pornstars` with `.thumb.cat` style cards (square thumbs, video count).
- `.filter-section` + `.sort-hd` for sort dropdown (alphabetical / popularity / video count — confirm on live HTML when implementing).
- Pagination `.page_nav` when not in search mode.

## In-page AJAX search

POST `/ajax/search_{search_type}` with JSON body `{ text }`.

- Replaces `.all_pornstars` HTML with result thumbs.
- Hides `.page_nav`, `.filter-section`, `.toptext-container` while searching.
- Variable `search_type` is embedded in page script on live site.

## Thumb data per pornstar

Typical fields in AJAX JSON items:

- `url`, `thumb`, `orig_name`, `count_videos`

## Linked examples (from mirrors)

`/pornstar/angela-white`, `/pornstar/gina-gerson`, `/pornstar/chloe-cherry`, etc. (~77 links in pornstars template).

## Profile pages

Separate page type — see `pornstar-profile.md`. Index only lists cards.

## Implementation priority

1. Add `GET /pornstars` + `PornstarsTemplate`.
2. Implement search AJAX + DB full-text or LIKE on name.
3. Wire sort query params to SQL `ORDER BY`.
