# Site map and URL patterns

Live site base: **https://pornsok.com**

## Page types (distinct templates / behaviors)

### 1. Home — paginated video feed

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/` | https://pornsok.com/ | Route + `index.html` mirror |
| `/{page}` | `/2` … `/1239` | **Missing** route; links exist in mirror |
| `/?sort=mv` | most viewed | Query handling **missing** |
| `/?sort=mc` | most commented | Query handling **missing** |
| `/?hd=1` | HD filter | Query handling **missing** |

**H1:** Top Trending Free Porn Videos  
**Sections:** weekly pornstars/channels/tags (AJAX refresh), watching now, main grid, pagination, SEO footer blurb.

### 2. Categories index

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/categories` | live | Route + `categories.html` |

**H1:** Porn Video Categories  
**Features:** genre search (`#search-genres-input` → POST `/ajax/search_cats_tags_queries`), grid `.all_cats`, top tags/pornstars blocks.

### 3. Pornstars index

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/pornstars` | live | Route + static mirror; AJAX search POST missing |

**H1:** Top Trending Pornstars  
**Features:** page search (`#search-page-input` → POST `/ajax/search_{type}`), sort/filter UI, `.all_pornstars` grid.

### 4. Channels index

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/channels` | live | **Missing** template + route |

**H1:** Top Trending Porn Channels

### 5. Category / tag listing (slug pages)

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/{slug}` | `/milf`, `/lesbian`, `/anal` | **Missing** (152+ distinct slugs linked from mirrors) |
| `/tags` | linked from homepage | **Missing** |

**H1 pattern:** `{NAME} - Latest Porn Scenes` (e.g. MILF)  
**Canonical:** `https://pornsok.com/{slug}`  
Typical size ~172 KB — same shell as home with filtered grid + sort + pagination.

### 6. Video detail

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/video/{slug}.html` | long kebab slug | **Missing** |

**H1:** video title  
**Features:** `#player_container2`, related videos, tags `.v-tags`, download links, comments + KEmoji, Schema.org `VideoObject`.  
**Size:** ~162 KB.
**Stream:** `/videofile/{token}` (base64 payload on live).
**Embed:** `/embeded/{slug}.html` (production spelling).

### 7. Channel profile

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/channel/{slug}` | `/channel/brazzers` | **Missing** |

**H1:** `{Channel} - Latest Videos`

### 8. Pornstar profile

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/pornstar/{slug}` | `/pornstar/angela-white` | **Missing** |

**H1:** `{Name} - Newest Videos`  
May include header banner (`#head-banner`, `#head-avatar`) on some entities.

### 9. Search

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/search?q=…` | redirects/rewrites | **Missing** |
| `/videos/{query}` | canonical for search | **Missing** |

Live example: `search?q=test` → title "Test Porn Videos…", canonical `https://pornsok.com/videos/test`.

### 10. Legal / static pages

| Pattern | Example | Local support |
|---------|---------|---------------|
| `/page/privacy.html` | Privacy Policy | **Missing** |
| `/page/dmca.html` | DMCA | **Missing** |
| `/page/terms.html` | Terms | **Missing** |
| `/page/2557.html` | 2257 | **Missing** |
| `/page/contact.html` | Contact | **Missing** |

### 11. AJAX (not user-facing pages)

See `03-frontend-javascript-behavior.md`. Required for header search, homepage carousels, in-page search on categories/pornstars.

## Link inventory from local `index.html` mirror

| Link type | Count (approx) |
|-----------|----------------|
| `/video/...` | 54 |
| category slug `/foo` | 39 |
| `/pornstar/...` | 23 |
| `/channel/...` | 23 |
| pagination `/2`… | 20 |
| nav + legal | 13 |

## Priority for replica (recommended)

1. **P0:** Static serving + home pagination + video detail + slug listings (core browse path).
2. **P1:** `/ajax/*` for search and homepage widgets (otherwise JS breaks UX).
3. **P2:** Pornstars/channels indexes and profiles.
4. **P3:** Legal pages, favorites, comments posting, emoji picker assets under `directory/style/`.
