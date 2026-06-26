# Performance and payload-budget smoke

Bead: **sok-replica.9.4**

This note records the performance and payload-budget smoke checks for the
replica's main page families. The goal is to **document baseline response time
and rendered HTML size** and to **flag only extreme regressions** — not to
enforce modern minimalism.

PornsOK pages are intentionally heavy: large thumb grids, inline SEO copy,
emoji sprite markup, and a sizable production JS bundle. The budgets here are
pragmatic ceilings that catch accidental explosions (runaway template loops,
duplicated layout, unbounded fan-out, or a static asset bloating by an order of
magnitude), while leaving wide headroom for the legitimately large production
layout.

## What the checks assert

For each main page (HTTP 200) the smoke test asserts:

1. the expected handler is hit (via the `X-Sok-Handler` marker)
2. rendered HTML is at least `MIN_HTML_BYTES` (4 KiB) — a smaller 200 means the
   template collapsed to a stub/error shell
3. rendered HTML is at most `MAX_HTML_BYTES` (6 MiB) — a larger payload means the
   render likely exploded
4. the in-process render+serve cycle finishes within `MAX_RESPONSE_TIME`
   (10 seconds) — a longer time means latency regressed sharply

A separate check serves `/static/js/main.min.js` and asserts it is non-empty,
under an 8 MiB ceiling, and served within the same time budget. This guards the
release static-serving path against latency or size regressions.

The budgets are intentionally generous. They are tripwires for pathological
regressions, not latency SLOs or page-weight targets. Observed baselines (below)
sit far inside every limit, so normal content growth will not flake the suite.

## Pages covered

| Path | Handler |
| --- | --- |
| `/` | `index` |
| `/categories` | `categories` |
| `/pornstars` | `pornstars` |
| `/channels` | `channels` |
| `/tags` | `tags` |
| `/milf` | `category_slug` |
| `/video/<sample>.html` | `video_html` |
| `/page/privacy.html` | `page_static` |
| `/static/js/main.min.js` | static (actix-files) |

## Run locally

```bash
cargo test --test performance_smoke -- --nocapture
```

`--nocapture` prints a `[perf-smoke]` line per page with the handler, status,
byte size, and measured time so baselines are visible in CI logs.

## 2026-06-26 baseline (this worktree)

Measured in the debug test profile against the live DB pool. Times include
shared-pool warmup and DB round-trips and will vary by machine and cache state;
the byte sizes are the stable signal.

| Path | Bytes | Size | Time |
| --- | --- | --- | --- |
| `/` | 111428 | 108.8 KiB | ~0.47 s |
| `/categories` | 98798 | 96.5 KiB | ~1.48 s |
| `/pornstars` | 93319 | 91.1 KiB | ~1.57 s |
| `/channels` | 88146 | 86.1 KiB | ~1.54 s |
| `/tags` | 81242 | 79.3 KiB | ~0.06 s |
| `/milf` | 95608 | 93.4 KiB | ~1.60 s |
| `/video/<sample>.html` | 104111 | 101.7 KiB | ~0.06 s |
| `/page/privacy.html` | 82078 | 80.2 KiB | ~0.001 s |
| `/static/js/main.min.js` | 125036 | 122.1 KiB | ~0.001 s |

All main pages land in the 79-109 KiB range and the JS bundle at ~122 KiB —
comfortably inside the documented ceilings, with the floor catching any future
collapse to a stub.
