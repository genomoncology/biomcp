---
priority: 2
---

# 2197 — Bound the supplement text cache

Status: OPEN.

Milestone: 0.9.3

## Outcome

The supplement text cache obeys the same age, size and disk limits as the rest of the cache, shows in `cache stats`, and reads without walking its directory.

- Evidence: release review of 7025e117: `sessions/article-supplement-text-v1` (from 2152) stores up to 16 MiB per file with no expiry; `cache clean --max-age` skips it, `cache stats` omits it, the size and disk limits ignore it, and each read walks the directory.
- Changes: bring it under the managed cache's limits and accounting, or store it through the existing HTTP cache path.
- Proof: outside-in cases: clean by age removes old entries, the size limit evicts them, stats count them, and a lookup reads one entry by key.

## Progress

- 2026-10-18 started
