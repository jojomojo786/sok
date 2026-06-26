# Homepage (`/`)

## Live reference

- **Evidence (2026-06-26):** `docs/raw/live-inventory-2026-06-26/home__desktop.png`, `docs/raw/live-inventory-2026-06-26/home__mobile.png`

- **URL:** https://pornsok.com/
- **Canonical:** `https://pornsok.com/`
- **Title:** Free Porn Videos & Hot Sex Movies | PornsOK.com
- **H1:** Top Trending Free Porn Videos
- **Local mirror:** `templates/index.html` (~202 KB)
- **Local route:** `GET /` → `IndexTemplate`

## SEO & meta

- `lang="en"`, `prefix="og: http://ogp.me/ns#"`, `data-theme="dark"` on `<html>`.
- Meta description emphasizes daily updates, pornstars/models.
- `link rel="next" href="https://pornsok.com/2"` for pagination SEO.
- RTA label meta: `RTA-5042-1996-1400-1577-RTA`.
- Open Graph: `og:type=website`, title/description/url/site_name, image `https://c.foxporn.tv/`.
- Favicons + `site.webmanifest` under `/static/`.

## Header (fixed `.header`, height 70px)

| Element | Classes / IDs | Notes |
|---------|---------------|-------|
| Logo | `.logotype` | SVG mark, links `/` |
| Primary nav | `.header-menu` | Home, Categories, Pornstars, Channels, etc. |
| Mega menu | `.nav` / `.nav-in` | Category tiles `.menu-pic`, `.menu-label`, `.sub-url` |
| Search | `.btn-search`, `.search-box`, `#main-search` | Expands over header; AJAX help |
| Theme | `#day-night`, `#day-night-icon` | Sun/moon SVG; toggles `data-theme` |
| Mobile | `.btn-mob`, `#side-panel` | Clone nav links |

Dropdown chevron on items with submenus: `i.d-menu`, `.link-cont`.

## Main content sections (top → bottom)

1. **This Week's Best Pornstars** (`#ajax_pornstars`) — refresh via `refresh_pornstars()`; link "All Pornstars" → `/pornstars`.
2. **This Week's Best Channels** (`#ajax_channels`) — `refresh_channels()`; → `/channels`.
3. **This Week's Best Porn Tags** (`#ajax_tags`) — `refresh_tags()`; → `/tags`.
4. **Videos Being Watched Now** (`#ajax_watching_now`) — refresh button with `.fa-refresh`.
5. **Primary grid** — `.thumbs-floats` with `.thumb.vid` items (33.33% width desktop).
6. **Pagination** — `.page_nav` / `.pagination` — pages 1–9, ellipsis, last (1239), Next.
7. **SEO text** — `section.desc-text` with marketing copy.

## Filters (`.filter-section`)

- Sort header `.sort-hd` with links to `/?sort=mv`, `/?sort=mc`, `/?hd=1` (seen in mirror).
- Placed in `.sect-title-wrap` next to H1 on listing pages; home includes filter row.

## Thumb card (`.thumb.vid`)

```html
<div class="thumb vid" itemscope itemtype="http://schema.org/ImageObject">
  <a class="thumb-in" href="/video/{slug}.html" target="_blank">
    <div class="thumb-img">
      <div class="video-preview"></div>
      <img class="thumb-cover" data-original="..." data-video="...mp4" />
      <div class="thumb-meta-top"><span class="ttime">...</span></div>
      <div class="thumb-meta-bottom">
        <span class="tview"><i class="fa fa-eye"></i>...</span>
        <span class="tlike"><svg>...</svg><span>N%</span></span>
      </div>
    </div>
    <div class="thumb-title" itemprop="name">...</div>
  </a>
</div>
```

- LazyLoad: placeholder `data:image/gif;base64,...` + `data-original`.
- Hover: shadow on `.thumb-title:before`, hide meta overlays, show preview video.
- Schema: `datePublished`, `InteractionCounter` for WatchAction / CommentAction.

## Footer

- `.footer` with responsive logo (`picture` sources at 949px breakpoint).
- Links: `/page/privacy.html`, dmca, terms, 2257, contact.
- `.footer-copyr` © 2026.

## Scripts

- Deferred `main.min.js`, `rocket-loader.min.js` (Cloudflare-style attributes on live).
- Globals documented in `03-frontend-javascript-behavior.md`.

## Backend data requirements

| Entity | Used for |
|--------|----------|
| Video | Main grid, watching now, AJAX carousels |
| Pornstar | Weekly best strip |
| Channel | Weekly best strip |
| Tag | Weekly tags strip |
| Pagination | Total pages (~1239 on live) |

## Replica notes

- `target="_blank"` on video links matches live.
- Numeric pagination route must not collide with category slugs — production uses bare `/2` for home pages only; slugs are textual.
- Implement `/ajax/update_*` before removing static HTML inside AJAX containers.
