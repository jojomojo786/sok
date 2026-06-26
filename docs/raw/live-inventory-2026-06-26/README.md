# Live inventory — 2026-06-26

- **Captured at (UTC):** `2026-06-26T06:40:22.749Z`
- **Tool:** `playwright` headless Chromium via `docs/raw/capture-live-inventory.mjs`
- **Pages:** home, categories, pornstars, channels, category `/milf`, sample video, channel `/channel/brazzers`, pornstar `/pornstar/angela-white`, search `?q=test`, legal `/page/privacy.html`
- **Viewports:** `desktop`, `mobile` (20 captures each: JSON DOM snippet + viewport screenshot)
- **Blocked pages:** none (all HTTP 200)
- **Search note:** request `https://pornsok.com/search?q=test` resolves to `https://pornsok.com/videos/test` in the browser.

See `manifest.json` for per-file paths.

## Reproduce

See `docs/raw/README.md` — `npm install` + `npx playwright install chromium` in `docs/raw`, then `npm run capture:live-inventory`.
