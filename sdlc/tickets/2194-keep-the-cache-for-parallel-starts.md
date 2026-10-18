---
priority: 2
---

# 2194 — Keep the cache for parallel starts

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 2c7d26c, accept

## Outcome

Parallel BioMCP processes starting on one cache use it, without warnings, whenever the cache epoch is already current.

- Evidence: release review of 7025e117: 12 parallel `get gene` runs on an empty cache, 10 bypassed the cache and each printed `WARN local HTTP cache initialization failed ... WouldBlock`; on a warm cache 2 of 12 still bypassed it. v0.9.2 showed neither.
- Cause: start-up takes an exclusive try-lock on the epoch lock file only to confirm the epoch is current (`try_ensure_body_limited_cache_epoch`, `src/cache/migration.rs:113-120`, called near `src/sources/mod.rs:1535`).
- Changes: `CHANGELOG.md`, `src/cache/migration.rs`, `src/sources/mod.rs`, `tests/test_http_cache_fail_soft.py`. confirm a current epoch without the exclusive lock (read the marker, or a shared lock), take the exclusive lock only to migrate, and keep contention during a real migration falling through quietly at debug level.
- Proof: a concurrency case driving the production start-up path (not the test-only blocking helpers) with many parallel starts on a current cache: every process uses the cache and none warns; red on current main.
