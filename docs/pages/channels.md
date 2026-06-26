# Channels index (`/channels`)

## Live reference

- **URL:** https://pornsok.com/channels
- **Canonical:** `https://pornsok.com/channels`
- **Title:** Free Porn Channels: List of Best Sex Channels | PornsOK.com
- **H1:** Top Trending Porn Channels
- **HTML size (fetch):** ~125 KB
- **Local:** no template file; linked from homepage mega menu and AJAX block

## Expected UI (from homepage channel strip)

- Same `.thumb.cat` or channel variant cards as pornstars.
- Channel examples: `/channel/brazzers`, `/channel/blacked`, `/channel/bang-bros-network`, `/channel/vixen`, etc.
- Refresh on home: `refresh_channels()` → `/ajax/update_channels`.

## Channel profile

See `channel-profile.md` for `/channel/{slug}` listing pages.

## Backend

- `channels` table: slug, title, thumb, video_count, optional network metadata.
- Listing sort + pagination analogous to pornstars index.

## Gap

Full gap — need mirrored template (curl save like other pages) and route.
