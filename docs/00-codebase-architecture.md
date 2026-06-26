# Codebase architecture

## Stack

| Layer | Choice | Location |
|-------|--------|----------|
| HTTP server | Actix Web 4 | `src/main.rs`, `src/handlers/mod.rs` |
| Templates | Askama 0.12 | `src/views/mod.rs`, `templates/*.html` |
| Database | SQLx 0.8 + MySQL | `src/db.rs`, `DATABASE_URL` |
| Static files | actix-files | configured in `main.rs` (serve `static/`) |
| Caching | moka (deferred) | See [caching-hot-listing-queries.md](./caching-hot-listing-queries.md); no handler wiring yet |
| Allocator | tikv-jemallocator | release builds |

## Entry point (`src/main.rs`)

- Loads `.env` via `dotenv`.
- Builds `Config` from `config.json` + env (`src/config.rs`).
- Creates MySQL pool (`DbPool`).
- Registers routes via `handlers::routes` and serves static assets.

## Routes (`src/handlers/mod.rs`)

**Dynamic HTML (Askama + `RenderContext`):**

| Method | Path | Handler | Template |
|--------|------|---------|----------|
| GET | `/` | `index` | `IndexTemplate` → `templates/index.html` |
| GET | `/categories` | `categories` | `CategoriesTemplate` → `templates/categories.html` |
| GET | `/pornstars` | `pornstars_list` | `PornstarsTemplate` → `templates/pornstars.html` |
| GET | `/health` | `health_check` | plain text `healthy` |

**Registered stubs** (plain `stub:…` text until dynamic pages land): `/channels`, `/tags`, `/{page}` numeric home pagination, `/video/{slug}.html`, `/channel/{slug}`, `/pornstar/{slug}`, `/page/{name}.html`, `/videos/{query}`, `/{slug}` category/tag fallback, `GET /ajax/*`.

Route precedence is tested (bead **sok-replica.2.1**). See [04-implementation-decisions.md](./04-implementation-decisions.md).

## Handler behavior

Listing handlers (`index`, `categories`, `pornstars_list`):

1. Run `sqlx::query("SELECT 1")` as a DB health probe.
2. Build `RenderContext` / `PageMeta` (`src/views/context.rs`) for title, canonical, H1, `rel=next`.
3. Render Askama templates (mirrored HTML still mostly static inside the shell).

Domain models live under `src/models/` (videos, taxonomy, pagination, comments). Catalog DDL: [catalog-schema.md](./catalog-schema.md).

## Configuration

- `config.json`: MySQL host/port/database/user (password empty in repo; use env).
- `.env.example`: `DATABASE_URL`, `BIND_ADDR=0.0.0.0:8080`.
- Local runbook: [local-development.md](./local-development.md) (**sok-replica.10.3**).

## Static assets

```
static/
  js/main.min.js          # site behavior (jQuery, lazy load, AJAX)
  js/rocket-loader.min.js # Cloudflare-style loader (from mirror)
  site.webmanifest
  fox-tpl/
    fonts/custom4/        # icomoon icon font
    images/               # footer-logo.svg, etc.
    js/                   # duplicate main.min.js path used in inline config
```

Templates reference CDN media: `https://c.foxporn.tv/fox-images/...` for thumbs and preview MP4s.

## Template strategy (current)

Templates are **full-page mirrors** of production HTML (~125–202 KB each):

| File | Bytes | Canonical (in HTML) |
|------|-------|---------------------|
| `index.html` | ~202 KB | `https://pornsok.com/` |
| `categories.html` | ~169 KB | `https://pornsok.com/categories` |
| `pornstars.html` | ~125 KB | `https://pornsok.com/pornstars` |

They include inline CSS (theme variables), header/footer, and hard-coded thumb grids. For an exact replica you will eventually:

1. Extract shared layout into Askama partials (`header.html`, `footer.html`, `thumb.html`).
2. Pass vectors of `Video`, `Category`, etc. into templates.
3. Keep class names and DOM structure **byte-for-byte compatible** with `main.min.js` selectors.

## Errors (`src/errors.rs`)

Central `AppError` for Actix responses (used by handlers).

## What the architecture implies

The repo has a **full route table** and **three mirrored listing templates** wired through shared render context; catalog queries, AJAX POST handlers, and remaining page templates are still in progress. Everything else (video pages, AJAX, search, pagination, channels) must be added to match live behavior documented under `docs/pages/` and `03-frontend-javascript-behavior.md`.
