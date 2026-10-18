---
priority: 1
---

# 2208 — Repair parallel cache reuse qualification

Status: OPEN.

Milestone: 0.9.3

## Outcome

Concurrent CLI starts reuse a fresh current-epoch cache while its epoch lock is held. Diagnose the current Linux release-qualification extra request and repair its cause without relaxing the no-refetch assertion, freshness, deadlines or cache safety.

- Evidence: full nonpublishing rehearsal 39131401809 on current main `7907ab4b0871922270718db6065a1be7c291431c`, qualification job 117687973670, `make test-python`, failed `tests/test_http_cache_fail_soft.py::test_parallel_current_cache_starts_use_cache_while_epoch_lock_is_held[command0]` at line257: `all parallel starts must reuse the cache`, request-log count `3 == 2`. Summary: 1 failed, 1281 passed, 8 skipped. Current main and its four coverage suites passed on the native builder; this Linux failure needs a cause, not a passing rerun.
- Changes: `tests/test_http_cache_fail_soft.py`, `src/cache/**`, `CHANGELOG.md`. Narrow the production claim after tracing the extra request. Restate the outcome and record the exact current-main focused reproduction before production edits. Preserve the existing production cache/replay test rather than adding a new cache or task mechanism.
- Proof: identify the actual extra provider request and why it bypassed or missed the populated key. Reproduce the relevant cause deterministically, make the real concurrent-process regression pass, preserve genuine fail-open/deadline behavior and ordinary cache-size enforcement, and obtain fresh review. Use occasional Linux build-job reproduction when necessary; routine development gates use the normal builder. Push source once ready, then use the shared landing gate and post-merge main check. Final full nonpublishing rehearsal must pass on the replacement candidate.

## Progress

- 2026-10-18 started
