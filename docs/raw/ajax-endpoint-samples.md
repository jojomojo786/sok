# AJAX endpoint samples (production)

Source: [https://pornsok.com](https://pornsok.com)
Captured (UTC): `2026-06-26T06:39:50Z`

Machine-readable aggregate: [ajax-endpoint-samples.json](./ajax-endpoint-samples.json).
Raw bodies: `*.body` + `*.meta.txt` in this directory.

## Notes

- Read-only POST samples; add_hit/favourite is analytics-only (empty 200).
- Client parses JSON via $.parseJSON for search_* and update_tags; other update_* inject HTML.

## Endpoints

### `POST /ajax/search_help`

- **HTTP status:** 200
- **Request:** `text=milf` (application/x-www-form-urlencoded)
- **Response:** application/json, 2423 bytes
- **Fetched:** 2026-06-26T06:39:54Z UTC

**Shape:**
- `search_text`: string echo of query
- `pornstars`: ['url_pornstar', 'name', 'orig_name', 'thumb', 'count_videos']
- `channels`: ['url', 'orig_name', 'rus_name', 'thumb', 'count_videos']
- `videos`: ['url', 'title', 'thumb', 'widethumb']

**Client:** `main_search` sends `text=` as urlencoded string (not JSON). Stale guard compares `search_text` to input.

### `POST /ajax/search_cats_tags_queries`

- **HTTP status:** 200
- **Request:** `text=milf` (application/x-www-form-urlencoded)
- **Response:** application/json, 6896 bytes
- **Fetched:** 2026-06-26T06:39:54Z UTC

**Shape:**
- `search_text`: string
- `items`: ['id', 'name', 'url']

**Client:** jQuery `data:{text:t}` → form body. Renders `items[].name` capitalized in `.tags-list`.

### `POST /ajax/update_pornstars`

- **HTTP status:** 200
- **Request:** `(empty POST body)` (none)
- **Response:** text/html fragment, 2653 bytes
- **Fetched:** 2026-06-26T06:39:54Z UTC

**Shape:**
- `root_elements`: ['.thumb.cat']
- `approx_thumb_count`: 30
- `uses_data_original`: False
- `uses_lazy_video`: False

**Client:** empty POST; HTML uses `.thumb.cat` tiles (same markup family as category thumbs).

### `POST /ajax/update_channels`

- **HTTP status:** 200
- **Request:** `(empty POST body)` (none)
- **Response:** text/html fragment, 2718 bytes
- **Fetched:** 2026-06-26T06:39:55Z UTC

**Shape:**
- `root_elements`: ['.thumb.cat']
- `approx_thumb_count`: 30
- `uses_data_original`: False
- `uses_lazy_video`: False

**Client:** empty POST; HTML uses `.thumb.cat` tiles with channel thumbs.

### `POST /ajax/update_tags`

- **HTTP status:** 200
- **Request:** `(empty POST body)` (none)
- **Response:** application/json, 5633 bytes
- **Fetched:** 2026-06-26T06:39:55Z UTC

**Shape:**
- `html`: string (tag list markup)
- `preload_before`: string URL prefix
- `preload_after`: string suffix
- `preload_array`: ['slug fragments']

**Client:** JSON with `html` plus `preload_before` / `preload_after` / `preload_array` for `process_preloads()`.

### `POST /ajax/update_watching_now`

- **HTTP status:** 200
- **Request:** `order_by=week_views` (application/x-www-form-urlencoded)
- **Response:** text/html fragment, 18595 bytes
- **Fetched:** 2026-06-26T06:39:55Z UTC

**Shape:**
- `root_elements`: ['.thumb.vid']
- `approx_thumb_count`: 84
- `uses_data_original`: False
- `uses_lazy_video`: True

**Client:** `data="order_by=week_views"`; replaces `#ajax_watching_now` with HTML (`.thumb.vid` grid).

### `POST /ajax/add_hit/favourite`

- **HTTP status:** 200
- **Request:** `(empty POST body)` (none)
- **Response:** empty, 0 bytes
- **Fetched:** 2026-06-26T06:39:55Z UTC

**Shape:**
- `note`: Zero-length body on HTTP 200; client ignores response

**Client:** `addFavorite()` — fire-and-forget analytics; then native bookmark fallback. **Not** a favorites list API.

## Blocked / edge cases (sampled)

| Case | Endpoint | Status | Observation |
|------|----------|--------|-------------|
| `text=a` (1 char) | `/ajax/search_help` | 200 | Still returns JSON groups (client only calls when length > 1). |
| `text=` empty | `/ajax/search_help` | 200 | Returns default/popular suggestions JSON (~2.3KB), not empty. |
| `text=a` | `/ajax/search_cats_tags_queries` | 200 | Returns many `items` (broad match). Client requires length > 1. |
| Authenticated favorites list | N/A | — | Not observed in `main.min.js`; only `add_hit/favourite` hit counter. |

