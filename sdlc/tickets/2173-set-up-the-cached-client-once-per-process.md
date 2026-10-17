---
priority: 2
---

# 2173 — Set up the cached client once per process

Status: OPEN.

Milestone: 0.9.3

Depends on: 2172

## Outcome

Parallel sources in one process share one cache setup: the cached client is built once per process, the per-call provider clients reuse it, and a source holding a shared cache lease makes maintenance in the same process wait within its deadline instead of failing.

- Evidence: found by the sweep after GitHub issue 292. File locks are per handle, so parallel sources in one process contend for `.body-limit-cache-v1.lock` and `.biomcp-operation.lock` like separate processes. `HTTP_CLIENT` (`src/sources/mod.rs:534-550`) is checked, then built, with no single-flight, so every parallel source runs full construction on a cold start. `semantic_scholar_provider_client` and `provider_url_client` (`src/sources/mod.rs:1681`, `1702`, `1722`, `1756`) rebuild the cached client on every call, retaking the epoch lock, the shared lock, the tree check and the start-up clean. The in-process lease registry (`src/cache/clear.rs:102-227`) returns its "cannot upgrade" WouldBlock through `?` at `src/cache/clear.rs:235` and `375` instead of waiting, so an upgrade from a legacy cache during a parallel search fails a source at once (`src/sources/mod.rs:1490`).
- Starts from: the client construction in `src/sources/mod.rs` and the lease registry in `src/cache/clear.rs`.
- Keeps: file locks for coordination between processes, deadlines, and per-provider client settings.
- Changes: claims `src/sources/mod.rs` `src/sources/tests/provider_network.rs` `src/cache/clear.rs` `tests/test_http_cache_fail_soft.py` `CHANGELOG.md`. Build the cached client behind an async once-cell. Derive the per-provider clients from it instead of rebuilding. Wait on an in-process shared lease within the caller's deadline.
- Proof: a test starts several sources together on a cold cache and counts one construction and one epoch-lock acquisition. On current main it counts one per source. A legacy-cache upgrade during a parallel search fails a source on current main and succeeds after the change.
- Defers: deadlines for the blocking locks on paths without one (`src/cache/clear.rs:211`, `316`; `src/cache/migration.rs:137`).

## Approved dependency batch and implementation brief

PM approved starting this lane on the restored 2172 candidate before that dependency lands, then reviewing and landing both as one batch. The restored cache implementation includes the fixes previously reviewed at 15f83e28e and the reversal of integration backout 1c3a0b072; current main beb08a9f3 is merged. Keep 2172 and 2173 individually reviewed. Do not push work in progress.

Restated outcome: one process performs shared cache construction once, concurrent sources reuse its result, provider-specific clients preserve their settings without rebuilding the cache, and local maintenance waits for a held shared lease only within its invocation deadline. Preserve request-path external contention bypass from 2172, cache security, explicit cancellation/deadlines, existing client-reset test behavior and per-provider authentication/rate limits.

The current restored 2172 code reproduces both stale article and search-all failures on published 75689e032 in job 1792259110228-83497. Diagnostics in 1792259226085-7162 identify concurrent article search and exact-keyword OLS4 construction: a losing constructor encounters the held epoch initialization lock, builds uncached and wins the global client slot, preventing the warm reply from reaching the cache. This is the existing shared-construction defect, not a new adapter or stale-note format defect. Before implementation, add the failing construction/lease regressions and record the exact red command and output on the current restored candidate.

The existing unchanged stale assertions must pass after the single construction fix. PM also requires an outside-in proof that a parallel cold search fills the cache and the second identical search is served from that cache. Keep the existing cache-failure CLI regressions. Request any further claim expansion before editing other files. All checks use build jobs; root coordinates Rust admission with the independent fixture and supplement lanes.
