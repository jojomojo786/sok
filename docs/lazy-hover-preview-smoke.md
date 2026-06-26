# Lazy-load and hover-preview smoke

Bead: **sok-replica.7.3**

This note records the local verification path for PornsOK-style thumb lazy loading and hover/touch preview behavior on the replica.

## Expected DOM contract

Representative `.thumb.vid` cards should render:

- a `.video-preview` overlay inside `.thumb-img`
- a `.thumb-cover` image with:
  - placeholder `src` (`data:image/gif;base64,...`)
  - `data-original` poster URL
  - `data-video` preview MP4 URL
- boot globals on home/listing pages:
  - `isTHUMBS_OR_PLAYER = true`
  - `directory = "/static/fox-tpl"`
  - `lazyThreshold = 2000`

Home template source: `templates/index.html`
Widget/AJAX fragments: `src/views/widgets.rs`, `src/views/slug_listing.rs`

## Expected `main.min.js` behavior

The replica keeps the production bundle at `static/js/main.min.js` and does not patch preview logic unless local asset paths break.

Relevant hooks:

- `myLazyLoad = new LazyLoad({ elements_selector: ".thumb-cover, .ke, .soc-img", data_src: "original", ... })`
- desktop: `mouseenter` on `.video-preview` injects `.video-preview__video` from sibling `.thumb-cover[data-video]`
- mobile: `touchstart` on `.video-preview` injects the same preview video and marks the overlay `.show`
- local replica serves deferred scripts through `static/js/rocket-loader.min.js`, which sets `window.__cfRLUnblockHandlers = true` before `main.min.js` runs

## Run locally

Start the app:

```bash
BIND_ADDR=127.0.0.1:8080 cargo run
```

Run the focused browser smoke from `docs/raw`:

```bash
cd docs/raw
npm install
npx playwright install chromium
SOK_BASE_URL=http://127.0.0.1:8080 node capture-lazy-hover-smoke.mjs
```

The script checks:

1. home page boot globals and representative card hooks
2. lazy-load resolution for a visible `.thumb-cover`
3. stable thumb layout after lazy-load (no major box-size jump)
4. desktop hover preview video injection
5. mobile tap-to-preview fallback

Artifacts are written to `docs/raw/lazy-hover-smoke-YYYY-MM-DD/`.

## Automated contract tests

Rust integration coverage lives in `tests/lazy_hover_preview.rs`:

- home HTML emits lazy/preview hooks
- `main.min.js` retains LazyLoad + hover/touch preview selectors
- `POST /ajax/update_watching_now` returns `.thumb.vid` fragments with `data-video`

Run:

```bash
cargo test --test lazy_hover_preview
```

## 2026-06-26 result

Local verification target: `http://127.0.0.1:8081/` (worktree `cargo run` on this branch)

Verified:

- home HTML emits placeholder `src` + `data-original` + `data-video` + `.video-preview`
- `main.min.js` boots `myLazyLoad` against `.thumb-cover` and injects hover/touch preview videos
- Playwright smoke passed for desktop hover preview and mobile tap fallback
- lazy-load resolution kept thumb box size stable on representative cards

No `main.min.js` patch was required for this issue.
