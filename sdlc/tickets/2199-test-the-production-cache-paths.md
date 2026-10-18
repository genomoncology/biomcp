---
priority: 2
---

# 2199 — Test the production cache paths

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision dc85d0e, accept

## Outcome

Cache tests drive the code users run.

- Evidence: release review of 7025e117: the blocking `migrate_http_cache*`, `ensure_body_limited_cache_epoch_until` and `build_http_client*` exist only under `#[cfg(test)]` and carry most cache tests, while the production `try_*` paths and `SharedHttpCache::initialize` have almost no direct tests. The parallel-start bypass fixed in 2194 lived in that gap. Start-up permission repair also stops once `.tree-secured-v1` exists.
- Changes: `src/cache/migration.rs` `src/sources/mod.rs` `src/sources/tests/provider_network.rs` `CHANGELOG.md`. Retire obsolete test-only duplicates and move retained cases onto production paths; preserve existing bounded startup-permission behavior.
- Proof: coverage of the production start-up, migration and client paths by named cases.

## Progress

- 2026-10-18 started

Coordinator pre-code scope: current production concurrency and fail-soft cases already cover the recent repairs. Remaining gap is blocking test-only migration/client construction used by legacy upgrade, contention, startup security and cost cases. Claim `src/cache/migration.rs`, `src/sources/mod.rs`, `src/sources/tests/provider_network.rs`, `src/cache/mod.rs` for exports only if needed, and `CHANGELOG.md`. Move retained cases to SharedHttpCache::initialize and try_* migration; remove unused blocking duplicates, preserve the actual fail-soft/no-partial-state/retry contract. Do not manufacture a wrong blocking assertion solely to produce RED. Show the existing test-only versus production-path gap and use meaningful behavior checks/mutations as needed. Keep the publication-fault observer seam where it still has consumers. Preserve bounded one-time recursive security repair and per-write security; untouched files broadened after the marker remain a stated limitation, not a new recursive scan. Keep existing Python fail-soft and persistent-size cases unchanged. Code may proceed after the after-release landing; build jobs wait for the initial2191 idle-host baseline.

Targeted job2199/1792349768360-28760 produced the meaningful production-initialize-None mutation RED in two migrated behavior cases, restored source, and passed90 of91 targeted cases. The remaining local-source-lease case exposed an incorrect migration expectation: production waits for registered leases owned by this process before making one external-lock attempt. Preserve that existing local pending/release contract through real SharedHttpCache::initialize and require Some(cache) afterward; external maintenance contention remains fail-soft. This is a test expectation correction, not a product defect or scope expansion. Reuse the90 passing cases and unchanged concurrent-constructor mutation failure, then rerun the corrected class and remaining Python/mechanical checks. Do not attribute the unchanged mutation run to a changed legacy assertion.

Final checks: job2199/1792349998584-35214 passed213.244s: corrected real-production local-lease case1pass,21Python cases passed22.29s, full mechanical lint/quality/advisory checks passed. Together with the applicable90 Rust passes and unchanged concurrent-initializer mutation RED in1792349768360-28760,91 distinct targeted Rust cases are green; no claim reuses that mutation for the subsequently changed legacy assertion. Developer stopped and pushed once at dc85d0e95f57e467b4927a9e3877f09b564c73b2; fresh reviewer accepted that code with no findings. Actual source claim excludes unchanged src/cache/mod.rs.
