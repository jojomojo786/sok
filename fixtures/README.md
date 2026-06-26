# Catalog dev/test fixtures (sok-replica.3.6)

Deterministic sample catalog data extracted from mirrored templates and AJAX samples.

- [`catalog_seed.json`](./catalog_seed.json) — canonical JSON fixture (embedded in the binary)
- [`../sql/seeds/dev_catalog.sql`](../sql/seeds/dev_catalog.sql) — MySQL seed script
- [`../src/fixtures/mod.rs`](../src/fixtures/mod.rs) — Rust loader (`load_catalog_seed`, `apply_catalog_seed`, `apply_default_catalog_seed`)

Sources: mirrored templates, `docs/raw/ajax-endpoint-samples.*`, `docs/pages/*` (see bead **sok-replica.3.6**).

For development/tests only; production ingestion remains separate.
