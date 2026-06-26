# Responsive navigation smoke

Focused Playwright capture for **sok-replica.7.4**: desktop header, mega menu hover, search expansion, mobile `.btn-mob` side panel, close overlay, and footer logo breakpoint on the local replica home page (`/`).

## Prerequisites

1. Local app running (health check: `curl http://127.0.0.1:8080/health` → `200`).
2. From repo root or worktree:

```bash
BIND_ADDR=127.0.0.1:8080 cargo run
```

3. Playwright deps in `docs/raw` (same as other captures):

```bash
cd docs/raw
npm install
npx playwright install chromium
```

## Run

```bash
cd docs/raw
SOK_BASE_URL=http://127.0.0.1:8080 node capture-responsive-navigation-smoke.mjs
```

Optional env:

| Variable | Default |
|----------|---------|
| `SOK_BASE_URL` | `http://127.0.0.1:8080` |
| `SOK_SEARCH_TERM` | `brazzers` (term fed to header autocomplete; must match seeded/db data) |

If another agent already holds `:8080` in this shared workspace, run the app on a
free port (e.g. `BIND_ADDR=127.0.0.1:8091 cargo run`) and pass the matching
`SOK_BASE_URL`.

Output directory: `docs/raw/responsive-navigation-smoke-YYYY-MM-DD/` with PNG screenshots, per-scenario JSON, and `manifest.json`. Exit code `1` if any check fails.

This output is a **regenerable local smoke artifact, not committed evidence**: it is
gitignored (`docs/raw/responsive-navigation-smoke-*/`), matching the existing
`local-browser-smoke-*` / `lazy-hover-smoke-*` convention. Each rerun rewrites
`captured_at`, the `base_url`/`url` port, and screenshot bytes, so the bytes are not
stable across runs. The committed, deterministic checks live in
`tests/responsive_navigation.rs`; the live-site reference under
`raw/live-inventory-2026-06-26/` is the committed parity evidence.

## Scenarios

| Artifact | What it proves |
|----------|----------------|
| `desktop__baseline.png` | Fixed 70px header, `.wrap` `padding-top: 70px`, desktop nav visible, footer `picture` 949/950px sources, `footer-logo.svg` selected at desktop width |
| `desktop__mega_menu_hover.png` | Hover on `.nav-show` → `.submenu-container.hovered`, menu drops at `top: 70px` (PornsOK mega menu delay ~600ms) |
| `desktop__search_expand.png` | `.btn-search.activate` → `.search-box.active`, header menu hidden, `#main-search` focused, header autocomplete fires `POST /ajax/search_help` → `200` |
| `mobile__baseline.png` | `.header-menu` hidden, `.btn-mob` visible, `#day-night` at `right: 80px`, `.footer-link` hidden ≤950px, no overlap among menu / theme / search controls |
| `mobile__side_panel_open.png` | `.btn-mob` → `#side-panel.active`, `#close-overlay` visible, cloned `.side-url` links present |
| `mobile__side_panel_close.png` | `.close-overlay` click closes panel and hides overlay |
| `mobile__search_expand.png` | Mobile search expand, `#main-search` focused, `POST /ajax/search_help` → `200`, no control overlap |

## Findings (2026-06-26)

All eight scenarios pass against a current build. Two earlier "search results
populated" failures from the partial Composer run were environmental, not
navigation defects:

1. The capture had hit a stale binary from another worktree (`~/sok-runtime`) that
   404s on `/ajax/*`; building this worktree's binary and binding a free port fixed it.
2. The probe term `milf` legitimately returns zero suggestions; `brazzers` (now the
   default `SEARCH_TERM`) returns a channel suggestion.

Known follow-up (out of scope for navigation): the mirrored `static/js/main.min.js`
calls `$.parseJSON(e)` on the autocomplete response, but jQuery 3.3.1 already parses
`application/json` responses to an object, so `$.parseJSON` throws
(`"[object Object]" is not valid JSON`) and `#search_result` never renders items. The
`/ajax/search_help` content type is intentionally `application/json` and locked by
`tests/ajax_search_help.rs` (sok-replica.5.1), so the smoke asserts the request/`200`
response rather than DOM items. Resolving live-parity rendering needs a separate
decision on either the AJAX content type or the mirrored client script.
| `mobile__footer_logo_breakpoint.png` | At 949px width, footer `currentSrc` uses `spacer.gif` per `picture` breakpoint |

## Parity reference

Committed live inventory snippets: [raw/live-inventory-2026-06-26/home__desktop.json](raw/live-inventory-2026-06-26/home__desktop.json), [home__mobile.json](raw/live-inventory-2026-06-26/home__mobile.json). Behavior notes: [03-frontend-javascript-behavior.md](03-frontend-javascript-behavior.md).

Static contract (no Playwright in CI): `cargo test responsive_navigation`.

## Related

- Broader page smoke: `capture-local-browser-smoke.mjs` / [raw/README.md](raw/README.md).
- Theme/mobile markup tests: `tests/theme_toggle.rs`.
