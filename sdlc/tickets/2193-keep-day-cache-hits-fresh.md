---
priority: 1
---

# 2193 — Keep day-cache hits fresh

Status: OPEN.

Milestone: 0.9.3

## Outcome

A day-cache hit inside its 24-hour window carries no stale note or warning; only a genuinely stale serve does.

- Evidence: release review of 7025e117: running `biomcp get drug imatinib` twice prints `Cache note: OpenFDA data served from cache, 3 s old (older than the provider's freshness window).` in the card, JSON `_meta.notes` and MCP replies, plus a stderr WARN. `search gwas -g KRAS` and `search all --gene BRAF --disease melanoma` do the same.
- Cause: `stamp_stale_serve_marker` (`src/cache/manager.rs` near 65-74) applies the provider's own freshness rules and ignores `day_cache_request_active()` and `DAY_CACHE_TTL`. No test covers a hit inside the day window.
- Changes: judge staleness for day-cached requests against the day window.
- Proof: recorded-reply cases for a hit inside the window (no note, no warning) and past it (note kept), for openFDA and GWAS; red on current main.
