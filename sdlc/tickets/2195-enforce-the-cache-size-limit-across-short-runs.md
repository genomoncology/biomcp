---
priority: 2
---

# 2195 — Enforce the cache size limit across short runs

Status: OPEN.

Milestone: 0.9.3

## Outcome

The cache size limit holds across many short CLI runs, as it did in v0.9.2, and the saved byte total cannot be torn.

- Evidence: release review of 7025e117: a 2.44 MB cache with `BIOMCP_CACHE_MAX_SIZE=2000000` grew to 2.46 MB after three `get gene` runs with no eviction; v0.9.2 evicted to 1.90 MB.
- Cause: from 2133, each process starts from the total in `.content-bytes-v1`, which only the daily clean and evictions update; ordinary writes never do. The marker is written in place, so a torn write reads as a smaller number.
- Changes: update the saved total after writes (or on exit) under the existing lock without reintroducing a tree walk, and replace the marker atomically.
- Proof: outside-in case: repeated short runs over the limit evict below it; a torn or missing marker recovers by recount; start-up stays walk-free.

## Progress

- 2026-10-18 started
