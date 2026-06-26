# Local development — run the Actix app

**Bead:** **sok-replica.10.3** (parent epic: **sok-replica.10** — configuration and ops).

This guide is for a **fresh agent or developer** cloning the repo: configure MySQL credentials, optionally apply catalog DDL and seed data, run the server, and verify `/health` and `/`.

## Prerequisites

| Requirement | Notes |
|-------------|--------|
| Rust toolchain | Stable; project uses Actix Web 4, Askama, SQLx 0.8 (MySQL). |
| MySQL 8+ | Reachable instance (team Aiven database `sok` or local MySQL). |
| Network | App connects to `DATABASE_URL` at startup; pool creation **panics** if the database is unreachable. |

## 1. Configure environment (no committed passwords)

1. From the repo root:

```bash
cp .env.example .env
```

If you already have a local `.env`, edit it instead of overwriting.

2. Set **`DATABASE_URL`** in `.env` (this **overrides** `config.json` database settings):

```bash
DATABASE_URL=mysql://USER:PASSWORD@HOST:PORT/sok
```

3. Optional listen address (default `0.0.0.0:8080`):

```bash
BIND_ADDR=127.0.0.1:8080
```

4. Alternative to embedding the password in `DATABASE_URL`: keep `config.json` `db.password` empty in git and export:

```bash
export DB_PASSWORD='your-password'
```

The loader builds a URL from `config.json` when `DATABASE_URL` is unset. See `src/config.rs` and `.env.example`.

**Security:** Never commit real passwords. `.env` is local-only (see `.gitignore`). `config.json` in the repo has an empty `db.password`.

**Config precedence** (`src/config.rs`):

- `DATABASE_URL` → database connection
- `BIND_ADDR` → bind address
- `DB_PASSWORD` → fills empty `config.json` `db.password` only when not using `DATABASE_URL`

## 2. Database schema (first-time or empty catalog)

The app does **not** run migrations automatically on `cargo run`. Apply DDL before relying on DB-backed listings.

Authoritative DDL:

- `migrations/0001_catalog_schema.sql` — full catalog (14 tables)
- `migrations/001_taxonomy.sql` — taxonomy alignment

Details: [catalog-schema.md](./catalog-schema.md).

**Option A — MySQL client (recommended if `sqlx-cli` is not installed):**

```bash
mysql -h HOST -P PORT -u USER -p sok < migrations/0001_catalog_schema.sql
mysql -h HOST -P PORT -u USER -p sok < migrations/001_taxonomy.sql
```

**Option B — sqlx-cli** (install separately: `cargo install sqlx-cli --no-default-features --features mysql`):

```bash
export DATABASE_URL='mysql://...'
sqlx migrate run
```

Note: only `0001_` and `001_` files exist under `migrations/` today; there is no `_sqlx_migrations` table unless you use sqlx migrate.

## 3. Fixture data and fallback behavior

Deterministic dev/test catalog lives in:

| Artifact | Purpose |
|----------|---------|
| `fixtures/catalog_seed.json` | Canonical JSON; **embedded** in the binary via `include_str!` |
| `sql/seeds/dev_catalog.sql` | MySQL seed (truncates catalog tables, then `INSERT`) |
| `fixtures/README.md` | Short pointer (bead **sok-replica.3.6**) |

**Load SQL seed** (after schema is applied):

```bash
mysql -h HOST -P PORT -u USER -p sok < sql/seeds/dev_catalog.sql
```

There is **no** standalone `cargo` binary to seed the DB. Rust APIs used in tests:

- `fixtures::ensure_dev_catalog_schema` — applies a **subset** of DDL (`sql/schema/videos.sql`, `migrations/001_taxonomy.sql`, entity schema SQL), not a full substitute for `0001_catalog_schema.sql`
- `fixtures::apply_default_catalog_seed` — schema helper + inserts from JSON

Integration test `apply_seed_populates_query_surfaces_when_database_available` in `src/fixtures/mod.rs` exercises `apply_default_catalog_seed` when `DATABASE_URL` is set.

**Runtime without seed:** Handlers such as `GET /` try the database first; on empty or failed queries they fall back to bundled JSON fixtures (`load_catalog_seed` / `seed_home_thumbs`). So **`/` can render without seeding**, as long as MySQL is **reachable** (pool connects at startup). For DB-backed behavior and AJAX that prefers live rows, apply schema + seed.

Optional dev reset when using the Rust seed path: `SOK_FIXTURES_RESET=1` drops catalog tables before `ensure_dev_catalog_schema` (see `src/fixtures/mod.rs`).

## 4. Build, test, and run

From repo root:

```bash
cargo test
cargo run
```

`src/main.rs` loads `.env` via `dotenv`, initializes structured logging (`src/logging.rs`), loads `Config`, creates the MySQL pool, and binds Actix on `BIND_ADDR`.

## 5. Verify routes

With the server running (default `http://127.0.0.1:8080` if you set `BIND_ADDR=127.0.0.1:8080`):

```bash
curl -sS http://127.0.0.1:8080/health
# Expect: HTTP 200 and body: healthy

curl -sS -D - http://127.0.0.1:8080/ -o /dev/null
curl -sS http://127.0.0.1:8080/ | head -c 500
# Expect: HTTP 200, text/html; home Askama template (PornsOK mirror shell)
```

Other useful smoke paths (dynamic HTML): `/categories`, `/pornstars`. Route map: [00-codebase-architecture.md](./00-codebase-architecture.md), [04-implementation-decisions.md](./04-implementation-decisions.md).

Integration tests in `tests/routes.rs` assert handler mapping (including reserved paths vs category slug fallback); they require a valid `DATABASE_URL` like `cargo run`.

## 6. Troubleshooting

| Symptom | Likely cause |
|---------|----------------|
| Process exits immediately on start with config error | Missing/invalid `DATABASE_URL` or password; see `.env.example` and error text from `Config::load()`. |
| Panic `Failed to create database pool` | Wrong credentials, firewall, or database down. |
| Home page works but counts look like static sample data | Empty DB — fixture fallback; run `sql/seeds/dev_catalog.sql` after migrations. |
| SQL errors on listing queries | Schema not applied — run `0001_catalog_schema.sql` and `001_taxonomy.sql`. |

## Related docs and beads

- [docs/README.md](./README.md) — documentation index
- [catalog-schema.md](./catalog-schema.md) — tables, slugs, migration notes
- [caching-hot-listing-queries.md](./caching-hot-listing-queries.md) — **sok-replica.10.4** (caching deferred)
- Epic **sok-replica** — replica roadmap; fixtures **sok-replica.3.6**
