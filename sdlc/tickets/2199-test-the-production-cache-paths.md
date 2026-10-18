---
priority: 2
---

# 2199 — Test the production cache paths

Status: OPEN.

Milestone: 0.9.4

## Outcome

Cache tests drive the code users run.

- Evidence: release review of 7025e117: the blocking `migrate_http_cache*`, `ensure_body_limited_cache_epoch_until` and `build_http_client*` exist only under `#[cfg(test)]` and carry most cache tests, while the production `try_*` paths and `SharedHttpCache::initialize` have almost no direct tests. The parallel-start bypass fixed in 2194 lived in that gap. Start-up permission repair also stops once `.tree-secured-v1` exists.
- Changes: retire test-only duplicates where possible and move their cases onto the production paths; decide whether start-up should repair broadened permissions again.
- Proof: coverage of the production start-up, migration and client paths by named cases.
