# Video detail (`/video/{slug}.html`)

## Live reference

Example: https://pornsok.com/video/dog-house-madi-collins-knows-more-sex-than-her-step-bro-decides-to-show-him-what-he-should-do.html

- **Canonical:** same path under `pornsok.com`
- **H1:** full video title
- **HTML size:** ~162 KB
- **Local:** not present in `templates/`

## Confirmed features (live HTML probe)

| Feature | Present |
|---------|---------|
| `#player_container2` (player mount; legacy `#player_container` not present in live DOM) | Yes |
| Related videos | Yes |
| `.v-tags` tag links | Yes |
| `.video_download` | Yes |
| `#more_video` / load more | Yes |
| `#leave_comment` | Yes |
| KEmoji comment UI | Yes |
| Schema.org VideoObject | Yes |

## Player JS globals

On video pages, inline script sets `isPLAYER = true` (vs `false` on thumbs-only listings). `isTHUMBS_OR_PLAYER` remains true.

## Thumb / media URLs

- Posters: `https://c.foxporn.tv/fox-images/videos/{slug}.jpg`
- Preview/teaser MP4: `m-{slug}.mp4` pattern on thumbs
- Full video path likely under `video_path` + CDN (streaming logic in player JS — extract when saving template)

## SEO

- Title: `{Video Title} | PornsOK` pattern
- Meta description from synopsis
- `datePublished`, duration (`PT###S`), interaction statistics

## Social / engagement

- Like percentage on thumbs (`tlike`) — video page likely has vote actions (check saved template).
- Comment count in schema on cards.

## Replica tasks

1. Save production HTML for one video into `templates/video.html` Askama shell.
2. Map DB fields: slug, title, duration_seconds, views, rating, tags[], channel, pornstars[], embed/url.
3. Implement comment POST + ammonia sanitization.
4. Related videos query (same tags/channel).
5. Download links table or generated signed URLs.

## Risk

Largest functional surface area after core listing; player must match CDN URL structure or proxy streams.

## Live inventory evidence (2026-06-26)

- Captured with headless Chromium (Playwright): desktop 1440×900, mobile 390×844.
- Manifest: `docs/raw/live-inventory-2026-06-26/manifest.json`
- Sample URL in capture matches doc example; snippet shows `id="player_container2"`.
- Evidence: `docs/raw/live-inventory-2026-06-26/video-sample__desktop.json`
