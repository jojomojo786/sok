# Frontend JavaScript behavior

Source: `static/js/main.min.js` (includes jQuery 3.3.1, LazyLoad, site-specific logic).

Inline boot variables (from `templates/index.html`):

```javascript
var isTHUMBS_OR_PLAYER = true,
    isPLAYER = false,
    lazyThreshold = 2000,
    directory = "/static/fox-tpl",
    thumbs_path = "https://c.foxporn.tv/fox-images/videos",
    thumbs_dir = "fox-images/videos",
    video_path = "video",
    seb = false,
    first_load = false,
    pjs_v = 17,
    screen_mode = 'n',
    is_mobile = false;
```

Video pages set `isPLAYER = true` (pattern on live site).

## Lazy loading

- **LazyLoad** on `img` with `data-original`, `data-video` for hover preview MP4.
- Class hooks: `.thumb-cover`, `.loading`, `.loaded`.
- Threshold ~2000px (config `lazyThreshold`).

## Thumb hover preview

- `.video-preview` overlay + `.video-preview__video` element.
- Uses `data-video` URL on thumb images.

## Header search (all pages with `#main-search`)

| Event | Behavior |
|-------|----------|
| input length > 1 | POST `/ajax/search_help` body `text={query}` |
| response | JSON with `pornstars`, `channels`, `videos` arrays |
| UI | `#search_result` dropdown; `createSearchItem()` builds rows |

Escape clears or closes; mobile toggles `.search-box.active`, hides `.header-menu`.

## Homepage AJAX refresh buttons

| Function | POST URL | Data | Target DOM |
|----------|----------|------|------------|
| `refresh_watching_now()` | `/ajax/update_watching_now` | `order_by=week_views` | `#ajax_watching_now` |
| `refresh_newest_videos()` | `/ajax/update_newest_videos` | `video_id`, `offset`, `count` | `#ajax_newest_videos` |
| `refresh_pornstars()` | `/ajax/update_pornstars` | (empty) | `#ajax_pornstars` |
| `refresh_channels()` | `/ajax/update_channels` | (empty) | `#ajax_channels` |
| `refresh_tags()` | `/ajax/update_tags` | (empty) | `#ajax_tags` (JSON with `html`, preload hints) |

Replica must return **HTML fragments** matching production structure (thumb blocks).

## Pornstars page search (`#search-page-input`)

- POST `/ajax/search_{search_type}` with `{ text: t }` — `search_type` is a server-rendered global on that page.
- Replaces `.all_pornstars` content; hides `.page_nav`, `.filter-section`, `.toptext-container` during search.
- Escape / `#search-page-x` restores prior content.

## Categories page search (`#search-genres-input`)

- POST `/ajax/search_cats_tags_queries` with `{ text: t }`.
- Injects tag list HTML into `#ajax_content`; hides `.all_cats` during search.

## Sorting UI

- `.sort-hd` click toggles `active` class (dropdown).
- Sort links use query params like `/?sort=mv` on home.

## Mobile side panel

- `.btn-mob` opens `#side-panel` + `#close-overlay`.
- Populated from `.head-url` and `.sub-url` in mega menu.

## Scroll / UX

- `#gotop` after 300px scroll.
- `scrollto()` accounts for fixed `.header` height (70px).
- `show_msg()` toast via `#msg_container`.

## Tooltips (`#tipbox`)

- `ShowVisualBox` / `HideVisualBox` for category/channel hover cards (image, optional HD badge, video count).

## Comments (video pages)

- **KEmoji** widget loads `directory/js/smiles_.json` (path under fox-tpl; may need mirroring).
- AJAX comment flows likely in other chunks of minified file — expect endpoints for post/list.

## Favorites

- `addFavorite()` POST `/ajax/add_hit/favourite` then browser bookmark fallback.

## Theme toggle (`#day-night`)

Production behavior is implemented in `static/js/main.min.js` (also mirrored at `static/fox-tpl/js/main.min.js`). The replica preserves CSS variables on `:root` and `[data-theme="dark"]`; the toggle only flips `data-theme` and icon state.

### Server-rendered default (all listing/detail templates)

| Piece | Default |
|-------|---------|
| `<html>` | `data-theme="dark"` (night UI) |
| `#day-night` | `title="Day mode"` (click switches toward day/light) |
| `#day-night-icon` | class `to-day`, `<use xlink:href="#sun-svg" />` |
| Inline boot | `screen_mode = 'n'` (`n` = night/dark, `d` = day/light) |
| SVG defs | `#sun-svg` and `#moon-svg` in page sprite block |

Video pages emit the same `screen_mode = 'n'` via `PlayerBootGlobals` (`src/views/player_media.rs`).

### Click handler contract

`$("#day-night").on("click", …)` branches on global `screen_mode` (not on a cookie read at load time):

| Current `screen_mode` | DOM / state after click | Cookie |
|----------------------|-------------------------|--------|
| `'d'` (day/light UI) | `data-theme="dark"`, `body` +`black`, sun icon, `to-day`, title `Day mode`, `screen_mode = 'n'` | `sc_mod=n` (~3650 days, `Path=/`) |
| `'n'` (night/dark UI) | remove `data-theme`, `body` −`black`, moon icon, `to-night`, title `Night mode`, `screen_mode = 'd'` | `sc_mod=d` |

### Persistence parity gap

- **Cookie only:** `set_cookie("sc_mod", …)` on each toggle. **No `localStorage`.**
- **No reload restore in bundle:** `main.min.js` does not call `get_cookie("sc_mod")` on load, so first paint always follows server HTML until the user clicks `#day-night`. Returning visitors with `sc_mod=d` still get dark markup until they toggle (same as mirrored production JS).

### Desktop / mobile markup

- Header includes `#day-night`, `#day-night-icon`, `.btn-mob` (hamburger), and sun/moon symbol defs on home, categories, pornstars, and video templates.
- Mobile CSS (`max-width: 950px`): `.btn-mob {display:block}`; `#day-night{ position: absolute; right: 80px; }` keeps the theme control visible beside the menu control.

Replica tests: `tests/theme_toggle.rs` (HTML + `main.min.js` string contract).

## Cookies

- `set_cookie` / `get_cookie` — used for theme and preload flags.

## Replica checklist for backend

1. Implement all POST `/ajax/*` routes with production-compatible JSON/HTML shapes.
2. Preserve element IDs: `ajax_watching_now`, `ajax_pornstars`, `ajax_channels`, `ajax_tags`, `main-search`, `search_result`.
3. Serve `static/fox-tpl/images/loadMoreVideos.gif` for loading states (referenced in JS); KEmoji needs `js/smiles_.json` and `style/rez/<cat>/emoji.png` under the same `directory` mount.
4. Keep `directory` path consistent or patch JS config.
5. For video pages, ensure player JS receives same globals (`isPLAYER`, related video arrays).

## Production AJAX samples (2026-06-26)

Captured from [pornsok.com](https://pornsok.com) (read-only POST). Full payloads: [docs/raw/ajax-endpoint-samples.json](../raw/ajax-endpoint-samples.json).

### `POST /ajax/search_help`

| Field | Type | Notes |
|-------|------|-------|
| `search_text` | string | Echo of query; client discards stale responses |
| `pornstars[]` | objects | `url_pornstar`, `name`, `orig_name`, `thumb`, `count_videos` |
| `channels[]` | objects | `url`, `orig_name`, `rus_name`, `thumb`, `count_videos` |
| `videos[]` | objects | `url`, `title`, `thumb`, `widethumb` (0 → CSS offset on thumb) |

Request body: urlencoded `text={query}` (jQuery passes a string, not JSON). Example query `milf` → HTTP 200, ~2.4KB JSON.

### `POST /ajax/search_cats_tags_queries`

| Field | Type | Notes |
|-------|------|-------|
| `search_text` | string | Echo for stale guard |
| `items[]` | objects | `id`, `name`, `url` (category/tag/search alias) |

Request: `text=milf` via form encoding. Example → HTTP 200, ~6.9KB JSON.

### `POST /ajax/update_pornstars` / `update_channels`

- Empty POST body.
- Response: **HTML fragment** (~2.6–2.7KB) injected into `#ajax_pornstars` / `#ajax_channels`.
- Markup uses `.thumb.cat` blocks (`thumb-in`, `thumb-cover`, `count-videos`, `thumb-title`).

### `POST /ajax/update_tags`

- Empty POST body.
- Response: **JSON** with `html` (tag list markup), `preload_before`, `preload_after`, `preload_array[]` (category mini image slugs).
- Example → HTTP 200, ~5.6KB.

### `POST /ajax/update_watching_now`

- Body: `order_by=week_views` (urlencoded string).
- Response: **HTML fragment** (~18KB) of `.thumb.vid` tiles for `#ajax_watching_now`.

### `POST /ajax/add_hit/favourite` (favorites / bookmark)

- Empty POST; HTTP **200** with **empty body**.
- Client does not read response; used as analytics before browser bookmark UI (`addFavorite()`).
- No user favorites list endpoint found in mirrored `main.min.js`.

### Uncertain / not sampled here

- Exact rotation algorithm for `update_*` widgets (server picks next catalog slice).
- Whether `search_help` with missing `text` param differs from `text=` (only empty string tested).
- Comment-related `/ajax/*` routes (separate minified regions).

## AJAX endpoints (consolidated list)

- `/ajax/search_help`
- `/ajax/search_cats_tags_queries`
- `/ajax/search_{type}` (pornstars page)
- `/ajax/update_watching_now`
- `/ajax/update_newest_videos`
- `/ajax/update_pornstars`
- `/ajax/update_channels`
- `/ajax/update_tags`
- `/ajax/add_hit/favourite`

### Video page (`isPLAYER = true`)

| Function / UI | POST URL | Data | Response |
|---------------|----------|------|----------|
| Infinite-scroll related pool | `/ajax/more_videos_3` | `videourl={slug}` | JSON array of related thumb batches |
| Show more related click | `/ajax/add_hit/more_videos` | (empty) | `ok` text |
| More comments | `/ajax/more_comments` | `videourl={slug}` | JSON `{ comments: "<html>" }` |
| Submit comment | `/ajax/comments` | `name`, `msg`, `vid` | JSON `{ result, text }` |
| Like / dislike | `/ajax/add_vote_v3` | `id_video`, `status` | JSON `{ raiting, msg }` (local stub; no DB vote write yet) |

Client sets a one-hour `video_{id}` cookie after a successful vote response; replica stub does not enforce server-side dedupe.
