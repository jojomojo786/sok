# docs/raw

Raw fetch samples and live-site evidence.

## Re-run live inventory capture

From `docs/raw` (creates **local** `node_modules/`, gitignored):

```bash
cd docs/raw
npm install
npx playwright install chromium
npm run capture:live-inventory
```

Output: `live-inventory-2026-06-26/` (or edit `capture-live-inventory.mjs` to use a new dated folder). Committed evidence is the JSON/PNG/manifest under `live-inventory-*`, not `node_modules/`.

## Run local browser smoke capture

Start the Actix app first, for example from the repo root:

```bash
BIND_ADDR=127.0.0.1:8080 cargo run
```

Then run the smoke capture from `docs/raw`:

```bash
cd docs/raw
npm install
npx playwright install chromium
SOK_BASE_URL=http://127.0.0.1:8080 npm run capture:local-browser-smoke
```

The script opens `/`, `/categories`, `/pornstars`, and the representative video page at desktop and mobile widths. It writes screenshots, snippets, check results, and a manifest to `local-browser-smoke-YYYY-MM-DD/`, comparing layout anchors against the committed `live-inventory-2026-06-26/` reference artifacts and failing on missing anchors, horizontal overflow, missing page-specific elements, or broken same-origin CSS/JS/image/font assets.

## Committed artifacts

| Path | Contents |
|------|----------|
| `live-inventory-2026-06-26/` | Playwright PNG + JSON snippets (desktop/mobile) |
| `live-fetch-summary.json` | Title/canonical/H1 per probed URL |
| `ajax-endpoint-samples.json` | Machine-readable POST samples |
| `ajax-endpoint-samples.md` | Human summary + client notes |
| `*.body` / `*.meta.txt` | Raw AJAX response bodies |

Synced from beads **sok-replica.1.1** / **sok-replica.1.3**; decisions index: [../04-implementation-decisions.md](../04-implementation-decisions.md).

## Run lazy-load / hover-preview smoke

With the app running on `http://127.0.0.1:8080`:

```bash
cd docs/raw
npm install
npx playwright install chromium
SOK_BASE_URL=http://127.0.0.1:8080 npm run capture:lazy-hover-smoke
```

This focused capture verifies home-page `.thumb-cover` lazy loading, `.video-preview` overlays, `data-video` preview URLs, desktop hover preview injection, and mobile tap fallback. See [../lazy-hover-preview-smoke.md](../lazy-hover-preview-smoke.md).

## Run responsive navigation smoke

With the app running (use a free port if `:8080` is taken in this shared workspace):

```bash
cd docs/raw
SOK_BASE_URL=http://127.0.0.1:8080 node capture-responsive-navigation-smoke.mjs
```

This verifies the desktop header, mega menu hover, search expansion, mobile `.btn-mob` side panel, close overlay, and footer logo breakpoint. See [../responsive-navigation-smoke.md](../responsive-navigation-smoke.md).
