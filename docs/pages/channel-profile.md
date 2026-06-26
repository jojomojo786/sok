# Channel profile (`/channel/{slug}`)

## Live reference

Example: https://pornsok.com/channel/brazzers

- **Canonical:** `https://pornsok.com/channel/brazzers`
- **Title:** Brazzers Porn Channel - Free Sex Videos | PornsOK.com
- **H1:** Brazzers - Latest Videos
- **HTML size:** ~176 KB

## UI elements

- May include branding banner (channel logo) — inspect saved HTML when mirroring.
- Video grid identical to category listings.
- Sort + pagination.
- Links from home mega menu and `#ajax_channels` widgets.

## Header banner pattern (site-wide CSS)

Templates define `#head-banner`, `#head-avatar`, `#head-name` for entity pages (pornstar/channel) with gradient overlay `#head-gradient`.

## Data

- `channels.slug`, `channels.title`, `channels.logo_url`
- Videos filtered by `channel_id`

## Search integration

Header AJAX search returns `channels` type with `url`, `orig_name`, `thumb`.

## Gap

No template or route in repo.
