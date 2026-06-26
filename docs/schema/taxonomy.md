# Taxonomy schema: categories and tags

## Table layout decision

**Categories and tags use separate tables** (`categories`, `tags`), not a single shared `taxonomy_terms` table.

### Rationale

| Concern | Separate tables | Single shared table |
|--------|-----------------|---------------------|
| Categories index (`/categories`) | Grid uses category thumbs at `fox-images/categories/{slug}.jpg`, intro copy, and stable sort order — category-specific columns stay on `categories`. | Extra nullable columns or JSON for kind-specific fields. |
| Tags | Homepage/widgets use weekly-view ranking; search AJAX returns compact name+URL rows without category-style square thumbs. | Same `kind` discriminator on every row; indexes less selective. |
| Slug URLs | Production uses one path `/{slug}` for both kinds; resolution is `find_category_by_slug` then `find_tag_by_slug` (documented in `taxonomy::resolve_listing_slug`). | One lookup, but mixed cardinality and different count refresh rules. |
| Video joins | `video_categories` and `video_tags` match how listing and detail docs describe filters. | Polymorphic `video_taxonomy(term_id, kind)` works but pushes kind checks into every join. |

A unified **search** surface still spans both: `search_categories_and_tags` unions categories and tags (plus optional `taxonomy_search_aliases`).

## Tables

- **`categories`** — slug (unique), display_name, description, thumb_url, video_count, intro_html, sort_order, is_active.
- **`tags`** — slug (unique), display_name, description, thumb_url, video_count, weekly_views, is_active.
- **`taxonomy_search_aliases`** — optional aliases for in-page search (`/ajax/search_cats_tags_queries`).
- **`video_categories`** — `(video_id, category_id)` many-to-many.
- **`video_tags`** — `(video_id, tag_id)` many-to-many.

DDL: [`migrations/001_taxonomy.sql`](../../migrations/001_taxonomy.sql).

## Query coverage (Rust: `src/models/taxonomy.rs`)

| Page / endpoint | Functions |
|-----------------|-----------|
| `/categories` grid | `list_categories_for_index`, `CategoryCard` |
| `/categories` search AJAX | `search_categories_and_tags`, `CatsTagsSearchItem` |
| `/{slug}` listing | `resolve_listing_slug`, `get_category_by_slug`, `get_tag_by_slug`, `list_video_ids_for_category`, `list_video_ids_for_tag` |
| Top viewed tags block | `list_top_viewed_tags` |
