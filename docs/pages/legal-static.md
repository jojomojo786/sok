# Legal and static pages (`/page/*.html`)

## URLs (footer on all major pages)

| Path | Live title (fetch) |
|------|-------------------|
| `/page/privacy.html` | Privacy Policy - Pornsok.com |
| `/page/dmca.html` | (DMCA — mirror for title) |
| `/page/terms.html` | Terms |
| `/page/2557.html` | 18 U.S.C. 2257 |
| `/page/contact.html` | Contact |

Example privacy:

- **Canonical:** `https://pornsok.com/page/privacy.html`
- **H1:** Privacy Policy - PornsOK.COM
- **HTML size:** ~112 KB (includes full site chrome)

## Layout

- Same global header/footer as listings (heavy inline CSS duplicated).
- Main content: legal prose in `.cont` / article-style blocks.
- `rel="nofollow"` on footer links to these pages.

## Replica approach

**Short term:** static Askama templates or `actix-files` from `templates/legal/`.

**Long term:** CMS table `pages` with `slug`, `title`, `body_html` sanitized.

## Compliance

- RTA meta on all pages.
- Cookie policy page linked as "Cookies Privacy Policy".

## Local gap

No routes; footer links 404 on replica server.
