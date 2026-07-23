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

Note: `capture:live-inventory` is the original browser evidence capture and truncates large snippets. Do not use it to refresh the production homepage `<main>` inventory used by Rust.

## Refresh embedded live source

The Rust binary embeds full-source captures at compile time with `include_str!`. The default refresh command updates the canonical embedded source directory, `live-source-2026-06-27/`, and the full homepage inventory JSON, `live-inventory-2026-06-26/home__desktop.json`.

Run a report-only drift check first:

```bash
cd docs/raw
SOK_PARITY_REPORT_ONLY=1 npm run refresh:live-source
```

If the report shows only `cloudflare_dynamic_only` or `unchanged_exact`, a normal refresh is safe:

```bash
cd docs/raw
npm run refresh:live-source
```

If normalized hashes changed, the script stops before writing. That is real content drift, not only Cloudflare byte churn. To intentionally accept it:

```bash
cd docs/raw
SOK_ACCEPT_CONTENT_DRIFT=1 npm run refresh:live-source
```

For a dry run that writes artifacts outside the repo, point both source and inventory outputs at temp paths:

```bash
cd docs/raw
SOK_REFRESH_SOURCE_DIR=/tmp/sok-live-source-test \
SOK_HOME_INVENTORY_PATH=/tmp/sok-home__desktop.json \
npm run refresh:live-source
```

After any real refresh, rebuild the Rust binary and rerun captured-source plus artifact parity. The source files are embedded at compile time, so a running or previously built binary will still serve the old bytes until rebuilt.

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

## Run byte parity comparison

Start the Actix app first, then compare live and local source bytes:

```bash
cd docs/raw
SOK_BASE_URL=http://127.0.0.1:8080 npm run compare:byte-parity
```

The verifier fetches live `pornsok.com` and the local app for `/`, `/categories`, `/pornstars`, `/channels`, `/page/privacy.html`, plus representative AJAX POST endpoints. It records status, content type, byte length, SHA-256, normalized SHA-256 diagnostics, title/H1/canonical, `.thumb.vid` / `.thumb.cat` counts, and an `exact_audit` classification explaining whether remaining non-exact bytes are normalized Cloudflare edge mutations, randomized AJAX samples, transport errors, or deterministic content drift.

By default it writes `local-byte-parity-YYYY-MM-DD.json` and exits non-zero when any live/local response differs exactly. Use `SOK_PARITY_REPORT_ONLY=1` for evidence capture without failing, `SOK_PARITY_OUTPUT=path.json` to choose the output file, and `SOK_PARITY_EXTRA_ROUTES=/video/example.html,/milf` to add GET probes.

To prove whether live bytes are stable before involving the local replica, run:

```bash
cd docs/raw
SOK_LIVE_INSTABILITY_SAMPLES=3 npm run evidence:live-instability
```

This repeatedly samples the same live GET and AJAX probes, then records unique raw SHA-256 counts, normalized SHA-256 counts, byte-length counts, and classifications such as `live_raw_dynamic_normalized_stable`, `live_random_six_card_sample`, and `live_stable_exact`. Use `SOK_LIVE_INSTABILITY_OUTPUT=path.json` to choose the output file.

To compare normal local public routes against the frozen full-source captures instead of fetching fresh live Cloudflare-mutated HTML, run:

```bash
cd docs/raw
SOK_BASE_URL=http://127.0.0.1:8080 npm run compare:artifact-parity
```

This reads `live-source-2026-06-27/manifest.json`, probes the local app's real GET routes, and reports exact plus normalized SHA-256 parity against the captured source files. It intentionally skips AJAX probes because the live `update_*` endpoints return independent random samples and there is no single stable captured byte target.

For frozen source replay verification, start the app with diagnostic routes enabled and run:

```bash
SOK_DIAG_ROUTES=1 BIND_ADDR=127.0.0.1:8080 cargo run
```

```bash
cd docs/raw
SOK_BASE_URL=http://127.0.0.1:8080 npm run compare:captured-source
```

This compares diagnostic route response bodies under `/_diag/source-replay/{label}` against files in `live-source-2026-06-27/`, and also checks captured random-widget bodies under `/_diag/source-replay/ajax/{name}` against `update_pornstars.body` / `update_channels.body`. It can prove byte-for-byte equality to specific captured source samples. It does not claim equality to a fresh live response, because live Cloudflare and AJAX bytes mutate between requests, and it does not validate the production dynamic rendering path. Use `SOK_REQUIRE_DYNAMIC=1` to also fetch the production route and require it to be distinct from diagnostic replay. Use `SOK_REPLAY_INCLUDE_AJAX=0` for GET-only diagnostic replay.

## Committed artifacts

| Path | Contents |
|------|----------|
| `live-source-2026-06-27/` | Full raw HTML source captures for replay-mode byte checks |
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
