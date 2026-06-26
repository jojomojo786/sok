# Search

## Live behavior

## Live redirect (2026-06-26)

Browser navigation to `https://pornsok.com/search?q=test` lands on **`https://pornsok.com/videos/test`** with canonical `https://pornsok.com/videos/test` and H1 `Test Porn Videos` (matches existing doc).

| Entry | Example | Result |
|-------|---------|--------|
| Header typeahead | `#main-search` | POST `/ajax/search_help` |
| Form submit | `.search-form` | navigates to search results (min 3 chars in JS guard) |
| Query URL | `/search?q=test` | **Title:** Test Porn Videos & Sex Scenes |
| Canonical | `/videos/test` | SEO canonical for term `test` |

## Header AJAX (`/ajax/search_help`)

**Request:** `POST`, body `text={query}`

**Response JSON** (structure from `makeResult` / `createSearchItem`):

- `search_text` — echo for stale-request guard
- `pornstars[]`: `url_pornstar`, `orig_name`, `thumb`
- `channels[]`: `url`, `orig_name`, `thumb`
- `videos[]`: `url`, `title`, `thumb`, `widethumb` (affects image CSS `top:-15%`)

Rendered as `<li class="pornstars|channels|videos">` in `#search_result > ul`.

## Full results page

- H1: `{Term} Porn Videos`
- Grid of `.thumb.vid` matching term
- Likely pagination — mirror via curl when implementing

## In-page searches

- Pornstars: `/ajax/search_{type}`
- Categories: `/ajax/search_cats_tags_queries`

## Replica

1. Implement unified search index (videos + stars + channels).
2. Debounce matches client-side (already in JS).
3. Map `/search` → `/videos/{slugified-query}` redirects like live.
4. Use `urlencoding` crate for canonical paths.

## Live inventory evidence (2026-06-26)

- Captured with headless Chromium (Playwright): desktop 1440×900, mobile 390×844.
- Manifest: `docs/raw/live-inventory-2026-06-26/manifest.json`
- `docs/raw/live-inventory-2026-06-26/search-test__desktop.json`
