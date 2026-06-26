# Category / tag listing (`/{slug}`)

## Live reference

Example: https://pornsok.com/milf

- **Canonical:** `https://pornsok.com/milf`
- **Title:** MILF Porn Videos - Watch Free Hot Mom Sex Scenes | PornsOK
- **H1:** MILF - Latest Porn Scenes
- **HTML size:** ~172 KB

## URL taxonomy

| Kind | Pattern | Examples |
|------|---------|----------|
| Category | `/{kebab-slug}` | `/milf`, `/lesbian`, `/big-dick` |
| Tags hub | `/tags` | linked from home |
| Home pagination | `/{integer}` | `/2` … **conflict risk** |

**Routing strategy for replica:** register numeric routes only for pages 2+ home OR use explicit `/page/{n}` internally with redirects; production uses bare integers for home pagination — study nginx rules from ops or mirror response headers.

## Page structure

- Same header/footer as home.
- H1 `{Category} - Latest Porn Scenes`.
- `.thumbs-floats` video grid (`.thumb.vid`).
- `.filter-section` / sort links (mv, mc, hd).
- `.page_nav` pagination within category.
- Optional `.toptext-container` SEO blurb at top.

## Query parameters

- `?sort=mv` most viewed
- `?sort=mc` most commented  
- `?hd=1` HD only

## Data model

```text
Category (or Tag)
  slug, display_name, description, thumb_url
VideoCategory (join)
  video_id, category_id
```

## Slug count

Home mirror alone links ~39 category slugs; categories index ~154 — full catalog larger.

## Local gap

No handler; all `/video` and `/{slug}` links from static HTML will 404 on localhost except `/` and `/categories`.
