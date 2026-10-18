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

## Progress

- 2026-10-18 started

Coordinator pre-code scope: current production concurrency and fail-soft cases already cover the recent repairs. Remaining gap is blocking test-only migration/client construction used by legacy upgrade, contention, startup security and cost cases. Claim `src/cache/migration.rs`, `src/sources/mod.rs`, `src/sources/tests/provider_network.rs`, `src/cache/mod.rs` for exports only if needed, and `CHANGELOG.md`. Move retained cases to SharedHttpCache::initialize and try_* migration; remove unused blocking duplicates, preserve the actual fail-soft/no-partial-state/retry contract. Do not manufacture a wrong blocking assertion solely to produce RED. Show the existing test-only versus production-path gap and use meaningful behavior checks/mutations as needed. Keep the publication-fault observer seam where it still has consumers. Preserve bounded one-time recursive security repair and per-write security; untouched files broadened after the marker remain a stated limitation, not a new recursive scan. Keep existing Python fail-soft and persistent-size cases unchanged. Code may proceed after the after-release landing; build jobs wait for the initial2191 idle-host baseline.
