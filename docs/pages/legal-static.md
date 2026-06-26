# Legal and static pages (`/page/*.html`)

## Routes

Replica serves these slugs from `src/views/legal_page.rs` via `templates/legal_page.html` and body fragments in `templates/legal/*_body.html`.

| Path | Live title | Live description | Live H1 |
|------|------------|------------------|---------|
| `/page/privacy.html` | Privacy Policy - Pornsok.com | Privacy Policy | Privacy Policy - PornsOK.COM |
| `/page/dmca.html` | DMCA Notice of Copyright Infringement Pornsok.com | DMCA Notice of Copyright Infringement Pornsok.com | DMCA Notice of Copyright Infringement Pornsok.com |
| `/page/terms.html` | Terms Of Service \| Pornsok.com | Terms and Conditions | Terms & Conditions - PornsOK.com |
| `/page/2557.html` | 18 USC 2257 Statement - Pornsok.com | 18 USC 2257 Statement - Pornsok.com | 18 USC 2257 Statement: Pornsok.com |
| `/page/contact.html` | Support and feedback - Pornsok.com | Feedback And Suggestions | Support and feedback |

## Layout and footer policy

- Shared site chrome: `templates/legal_page.html` (same header/footer pattern as listings).
- Main content shell: `<section class="desc-text page-text">` inside `.cont`.
- Footer legal links on all major pages use `rel="nofollow"` and point to the five slugs above.
- Footer copyright year comes from `SiteLayout::copyright_year` (`ctx.layout.copyright_year` in templates).

## Content sources (2026-06-26)

Live HTML captures used to mirror static body fragments:

| Slug | Source URL | Local capture |
|------|------------|---------------|
| privacy | https://pornsok.com/page/privacy.html | `docs/raw/live-legal-2026-06-26/privacy.html` |
| dmca | https://pornsok.com/page/dmca.html | `docs/raw/live-legal-2026-06-26/dmca.html` |
| terms | https://pornsok.com/page/terms.html | `docs/raw/live-legal-2026-06-26/terms.html` |
| 2557 | https://pornsok.com/page/2557.html | `docs/raw/live-legal-2026-06-26/2557.html` |
| contact | https://pornsok.com/page/contact.html | `docs/raw/live-legal-2026-06-26/contact.html` |

Earlier inventory snippet for privacy only: `docs/raw/live-inventory-2026-06-26/legal-privacy__desktop.json` (truncated `main` field).

## Replica static choices

- **Privacy / DMCA / Terms / 2257:** body HTML copied from live `<section class="desc-text page-text">` (including Cloudflare obfuscated email markup where present).
- **Contact:** live textarea + Send button markup is preserved for visual parity; the live `send_msg()` AJAX script is omitted and the Send button is non-functional in the replica (`onclick="return false;"`). Live posts to `/ajax/send_msg`.
- **Terms h1 escaping:** live emits a literal `&` in `<h1>Terms & Conditions - PornsOK.com</h1>`. The replica renders the same text through Askama auto-escaping, so the served bytes are `&amp;` (identical when displayed). Tests assert the escaped form.

## Compliance

- RTA meta on all pages.
- Cookie policy page linked in footer as "Cookies Privacy Policy".

## Tests

- `tests/legal_static_pages.rs` — 200 responses, canonical/title/h1 needles, nofollow footer links, shared body shell.
- `tests/routes.rs` and `tests/metadata.rs` — route mapping and head meta for privacy sample.
