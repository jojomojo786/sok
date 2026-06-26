# Agent handoff validation checklist

Bead: **sok-replica.9.5**

Run this checklist before closing any implementation bead. It is
command-oriented and ordered cheapest-first: fast compile checks, then focused
Rust suites, then the browser/performance smokes that need a running server,
then the Beads close protocol. Skip a server-only smoke only when your change
cannot affect it, and say so in the handoff.

Server smokes assume the app is running locally:

```bash
BIND_ADDR=127.0.0.1:8080 cargo run
```

Use `http://127.0.0.1:8080` as the base URL below (or whatever `BIND_ADDR` you
bound). Some Rust integration tests use a live DB pool, so keep your `.env`
configured per [local-development.md](./local-development.md).

## 1. Build and full test suite

```bash
cargo build
cargo test
```

`cargo build` must compile clean. `cargo test` runs every integration target in
`tests/` against the configured pool. If you only touched a narrow area, still
run the full suite once before handoff.

## 2. Focused route / static / AJAX tests

Run the targets that cover the surface you changed (each is a file under
`tests/`):

```bash
cargo test --test routes          # route table + handler markers
cargo test --test static_assets   # /static/* serving + headers
cargo test --test ajax_contracts  # /ajax/* request/response contracts
```

Related focused targets worth running when relevant: `home`, `metadata`,
`metadata_assets`, `search_results`, `pagination`, `category_slug_listing`,
`legal_static_pages`, `lazy_hover_preview`. List the full set with:

```bash
cargo test --test  # then Tab, or:
ls tests/*.rs
```

## 3. Browser smoke

With the server running, run the local browser smoke from `docs/raw`:

```bash
cd docs/raw
npm install
npx playwright install chromium
SOK_BASE_URL=http://127.0.0.1:8080 node capture-local-browser-smoke.mjs
```

For lazy-load / hover-preview changes also run the focused capture and its Rust
contract test (see [lazy-hover-preview-smoke.md](./lazy-hover-preview-smoke.md)):

```bash
SOK_BASE_URL=http://127.0.0.1:8080 node capture-lazy-hover-smoke.mjs
cargo test --test lazy_hover_preview
```

Artifacts land in `docs/raw/*-smoke-YYYY-MM-DD/`; eyeball them before handoff.

## 4. Performance / payload smoke

```bash
cargo test --test performance_smoke -- --nocapture
```

`--nocapture` prints a `[perf-smoke]` line per page (handler, status, byte size,
time). Confirm each page stays inside the documented ceilings; details and
baselines in [performance-payload-smoke.md](./performance-payload-smoke.md).

## 5. Beads close protocol

Only after the checks above pass:

```bash
git status                       # confirm intended files only
bd show <id>                     # re-read acceptance criteria
bd close <id> --reason="..."     # close the implemented bead
```

Do not commit or push, or run `bd dolt push`, unless the active profile or the
current request authorizes it (see `AGENTS.md` → Beads / Session Completion).
File follow-up beads for anything left out of scope. In the handoff, report
changed files, the validation commands you ran with results, and any check you
skipped and why.
