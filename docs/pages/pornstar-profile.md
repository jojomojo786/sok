# Pornstar profile (`/pornstar/{slug}`)

## Live reference

Example: https://pornsok.com/pornstar/angela-white

- **Canonical:** `https://pornsok.com/pornstar/angela-white`
- **Title:** Angela White Free Porn Videos and Scenes | PornsOK.com
- **H1:** Angela White - Newest Videos
- **HTML size:** ~177 KB

## UI

- Optional profile header (`#head-banner`, avatar image, verified badge SVG in `#head-name`).
- Video grid `.thumb.vid` with same metadata as home.
- Sort/filter/pagination consistent with channel pages.

## Data

- `pornstars`: slug, display_name, thumb, optional bio, video_count
- Join table `video_pornstars`

## Indexes

Linked from:

- Home AJAX `#ajax_pornstars`
- `/pornstars` grid
- Categories page "top viewed pornstars"

## Gap

No route; 23+ pornstar URLs on home mirror alone.
