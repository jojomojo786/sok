# Manifest, favicon, and social metadata assets

## Local asset URLs

Mirrored HTML references root metadata assets served from `static/` via `configure_static` in `src/lib.rs`:

- `/favicon.ico`
- `/apple-touch-icon.png`
- `/favicon-32x32.png`
- `/favicon-16x16.png`
- `/safari-pinned-tab.svg`
- `/site.webmanifest`
- `/android-chrome-192x192.png` and `/android-chrome-512x512.png` (manifest icons only)

## Head tags (main pages)

Templates emit favicon links, `rel="manifest"`, Safari pinned tab, `theme-color`, MS tile color, and Open Graph `og:image` pointing at the deliberate CDN placeholder `https://c.foxporn.tv/` (not a local file).

## Verification

Integration coverage:

- `tests/metadata_assets.rs` — head tag snapshot + HTTP 200 for every local metadata asset referenced on main page families (home, taxonomy, entity, search, legal, sample video), plus manifest JSON and icon resolution.
- `tests/static_assets.rs` — critical root icon paths and page-referenced local assets.
- `tests/metadata.rs` — cross-family OG and description tags.

Run:

```bash
cargo fmt --check
cargo test --test static_assets --test metadata --test metadata_assets
```
