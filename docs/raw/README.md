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
