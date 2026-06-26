# Askama partial candidates (mirrored templates)

Research for **sok-replica.1.2**: decompose `templates/index.html`, `templates/categories.html`, and `templates/pornstars.html` into reusable Askama partials without breaking `static/js/main.min.js`.

**Scope:** documentation only — production templates were not edited.

**Related:** [03-frontend-javascript-behavior.md](./03-frontend-javascript-behavior.md)

---

## Template overlap summary

| Block | index | categories | pornstars | Partial candidate |
|-------|:-----:|:----------:|:---------:|-------------------|
| Inline `<head>` CSS + meta shell | ✓ | ✓ | ✓ | `partials/head_base.html` + page vars |
| SVG `<symbol>` sprite | ✓ | ✓ | ✓ | `partials/svg_sprite.html` (+ optional symbols) |
| `<header>` | ✓ | ✓ | ✓ | `partials/header.html` |
| `<footer>` | ✓ (slight diff) | ✓ | ✓ | `partials/footer.html` |
| Toast `#msg_container` | ✓ | ✓ | ✓ | `partials/msg_toast.html` (byte-identical) |
| Boot `<script>` + `main.min.js` | ✓ | ✓ | ✓ | `partials/scripts_boot.html` |
| Sort / filter UI | ✓ | — | ✓ | `filter_sort_home.html`, `filter_sort_pornstars.html` |
| In-page search box | — | ✓ | ✓ | `search_box_categories.html`, `search_box_pornstars.html` |
| Thumb cards | ✓ | ✓ | ✓ | macros in `partials/macros/thumb.html` |
| Pagination | ✓ | — | ✓ | `partials/pagination.html` |
| Homepage AJAX rails | ✓ | — | — | `partials/home_ajax_rails.html` |

Suggested layout:

```text
layout.html
  {% include "partials/head_base.html" %}
  <body>
    {% include "partials/svg_sprite.html" %}
    <div class="wrap">
      {% include "partials/header.html" %}
      <main> … page body … </main>
      {% include "partials/footer.html" %}
    </div>
    {% include "partials/msg_toast.html" %}
    {% include "partials/scripts_boot.html" %}
  </body>
</html>
```

Page templates keep **main** content and page-only inline scripts (sort dropdown handlers, `search_type` on pornstars).

---

## 1. `partials/head_base.html`

**Source:** `<head>…</head>` (~51–57 KB inline CSS per file).

**Preserve**

- `html` attrs: `lang="en"`, `prefix="og: …"`, `data-theme="dark"`.
- Askama SEO: `ctx.page.title`, `description`, `canonical_href()`, `rel_prev` / `rel_next`, `og_*`.
- Normalize asset paths (`/static/fox-tpl` vs mirror typos like `https:/fox-tpl`).

---

## 2. `partials/svg_sprite.html`

**Source:** hidden root `<svg>` with `<symbol id="…-svg">` before `<header>`.

**Preserve symbol `id` values** (used by `xlink:href="#…"`):

| Symbol `id` | index | categories | pornstars |
|-------------|:-----:|:----------:|:---------:|
| `camera-svg` | ✓ | ✓ | ✓ |
| `thumb-up-svg` | ✓ | — | — |
| `menu-svg` | ✓ | ✓ | ✓ |
| `vk-svg`, `telegram-svg`, `fb-svg` | ✓ | ✓ | ✓ |
| `search-svg` | ✓ | ✓ | ✓ |
| `moon-svg`, `sun-svg` | ✓ | ✓ | ✓ |
| `film-svg` | ✓ | — | — |
| `cats-svg` | — | ✓ | — |
| `star-svg` | — | — | ✓ |

Use a shared base plus conditional blocks for page-only symbols.

---

## 3. `partials/header.html`

**Source:** `<header class="header">…</header>` (~25 KB; markup differs slightly by page but same hook surface).

**IDs (required by `main.min.js`)**

| ID | Role |
|----|------|
| `main-search` | Header typeahead → POST `/ajax/search_help` |
| `search_result` | Dropdown; JS uses `#search_result > ul` |
| `searching` | Search UI companion |
| `day-night` | Theme control |
| `day-night-icon` | Icon; classes `to-night` / `to-day` |
| `menu-top-list` | Mega-menu list anchor |
| `add-fav`, `porn-dude` | Bookmark / promo |

**Classes (JS / layout)**

| Class | Role |
|-------|------|
| `.header`, `.header-in`, `.header-menu` | Fixed header; `scrollto()` uses ~70px |
| `.logotype` | Mobile search adds `.mobile` |
| `.btn-search`, `.activate`, `.submit-search`, `.x-search` | Search expand/collapse |
| `.search-box`, `.search-box.active`, `.search-form`, `.search-input` | Search layout |
| `.head-url`, `.sub-url` | Cloned into `#side-panel` on DOM ready |
| `.btn-mob` | Opens mobile panel |
| `.nav`, `.nav-in`, `.nav-show` | Mega menu |

---

## 4. `partials/footer.html`

**Source:** `<footer class="footer">…</footer>` (categories ≡ pornstars; index differs slightly in logo `src`).

**Preserve:** `.footer`, `.footer-in`, `.footer-menu`, `.footer-link`, `.footer-copyr`; legal links `/page/privacy.html`, `dmca`, `terms`, `2557`, `contact` with `rel="nofollow"`.

---

## 5. `partials/msg_toast.html`

Identical on all three pages:

```html
<div id="msg_container" class="notice-wrap">…<p id="text_msg"></p>…</div>
```

**Preserve:** `.notice-wrap`, `.notice-item-wrapper`, `.notice-item`; `show_msg()` targets `#msg_container` and `#text_msg`.

---

## 6. `partials/scripts_boot.html`

**Preserve globals:** `isTHUMBS_OR_PLAYER`, `isPLAYER`, `lazyThreshold`, `directory`, `thumbs_path`, `thumbs_dir`, `video_path`, `pjs_v`, `is_mobile`, optional `preload_thumbs`, page `search_type` (`'pstars'` on pornstars).

**Preserve:** `<script src="/static/js/main.min.js" defer>`.

| Page | `isTHUMBS_OR_PLAYER` | `directory` in mirror |
|------|----------------------|------------------------|
| index | `true` | `"/static/fox-tpl"` |
| categories | `false` | `"https:/fox-tpl"` (typo) |
| pornstars | `false` | `"https://c.foxporn.tv/fox-tpl"` |

Replica should emit one canonical `directory` (e.g. `/static/fox-tpl`).

---

## 7. Filter / sort partials

### `partials/filter_sort_home.html` (index)

**Wrapper:** `.filter-section`.

| Hook | Purpose |
|------|---------|
| `#show_sort` | Mobile sort button |
| `#showBtnSort` | Desktop dropdown; JS adds `.active` |
| `.filter-container`, `.filter-container.deployed` | Mobile expand |
| `.qty_sort` | All / HD links |
| `.select_sort` | Dropdown panel |
| `.sort-hd` | Generic toggle in `main.min.js` |

Inline script uses: `#show_sort`, `.filter-container`, `#showBtnSort`, `.select_sort`, `.deployed`.

### `partials/filter_sort_pornstars.html`

Same hooks as home plus **`.filter-section.pornstars`**. Inline script sets `var search_type = 'pstars';`.

---

## 8. In-page search partials

### `partials/search_box_categories.html`

| Hook | JS |
|------|-----|
| `#search-box-page` | Wrapper |
| `#search-genres-input` | POST `/ajax/search_cats_tags_queries` |
| `#search-genres-x` | Clear |

**Side effects:** hides `.all_cats`, `.toptext-container`; fills `#ajax_content`.

### `partials/search_box_pornstars.html`

| Hook | JS |
|------|-----|
| `#search-box-page` | Wrapper |
| `#search-page-input` | POST `/ajax/search_{search_type}` |
| `#search-page-x` | Clear |

**Side effects:** replaces `.all_pornstars`; hides `.page_nav`, `.filter-section`, `.toptext-container`. Injected rows use `.thumb.cat`, `.thumb-in`, `.count-videos`, `#camera-svg`.

---

## 9. Thumb macros (`partials/macros/thumb.html`)

### `thumb_cat` (category / pornstar / channel)

- Outer: `<div class="thumb cat" itemscope itemtype="http://schema.org/ImageObject">`
- **Preserve:** `.thumb`, `.thumb.cat`, `.thumb-in`, `.thumb-img`, `.thumb-cover`, `.thumb-title`, `.count-videos`, `itemprop` attrs, `xlink:href="#camera-svg"`
- Lazy: `data-original` + placeholder (index/pornstars); categories often direct `src`

### `thumb_vid` (index video grid)

- Outer: `<div class="thumb vid" …>`
- **Preserve:** `.video-preview`, `.thumb-meta-top`, `.thumb-meta-bottom`, `.ttime`, `.tview`, `.tlike`, `data-video`, `#thumb-up-svg` on index

### Listing containers (keep on page or section partial)

| Container | Pages | JS |
|-----------|-------|-----|
| `.thumbs-floats` | index | layout |
| `.all_cats` | categories | genre search hides |
| `.all_pornstars` | categories (related), pornstars grid | pornstar search replaces HTML |
| `#ajax_content` | categories | search results |
| `#ajax_pornstars`, `#ajax_channels` | index | `refresh_pornstars/channels` |
| `#ajax_tags` | index | `refresh_tags` |
| `#ajax_watching_now` | index | `refresh_watching_now` |

---

## 10. `partials/pagination.html`

**Preserve:** `.page_nav`, `.pagination`, `.active`, `.pag-num`, `.dots`, `.next`, `rel="next"` where present.

Absent on `categories.html` main listing in the current mirror.

---

## 11–13. Page body partials (candidates)

**Index:** `home_toptext` (`.toptext-container`), `home_video_grid` (`.thumbs-floats`), related sections with `#ajax_pornstars`, `#ajax_channels`, `#ajax_tags`, `#ajax_watching_now`, `home_desc_footer` (`.desc-text`), then `pagination`.

**Categories:** title row + `search_box_categories`, empty `#ajax_content`, `categories_grid` (`.all_cats`), optional `.desc-text`.

**Pornstars:** `sect-title-wrap` + filter + `search_box_pornstars`, `.all_pornstars` grid, `pagination`. Keep page-inline sort script with `search_type`.

---

## DOM hook checklist (templates ∩ `main.min.js`)

**IDs:** `#main-search`, `#search_result`, `#searching`, `#day-night`, `#day-night-icon`, `#msg_container`, `#text_msg`, `#ajax_watching_now`, `#ajax_pornstars`, `#ajax_channels`, `#ajax_tags`, `#ajax_content`, `#search-genres-input`, `#search-genres-x`, `#search-page-input`, `#search-page-x`, `#show_sort`, `#showBtnSort`, `#search-box-page`

**Classes:** `.header`, `.header-menu`, `.logotype`, `.btn-search`, `.search-box`, `.search-form`, `.head-url`, `.sub-url`, `.btn-mob`, `.page_nav`, `.filter-section`, `.filter-container`, `.select_sort`, `.sort-hd`, `.all_pornstars`, `.all_cats`, `.toptext-container`, `.notice-item`, `.thumb`, `.thumb-in`, `.video-preview`, `.pagination`

**JS-injected (do not duplicate in static partials):** `#gotop`, `#tipbox`, `#side-panel`, `#close-overlay` (appended in `$(document).ready`).

---

## Extraction order

1. `msg_toast`, `footer`, `svg_sprite`
2. `header` (search + mobile menu)
3. `scripts_boot` + normalize `directory`
4. Thumb macros (AJAX fragments)
5. `pagination`, filter/search partials
6. Page-specific main sections
