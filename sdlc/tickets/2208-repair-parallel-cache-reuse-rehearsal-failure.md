---
priority: 1
---

# 2208 — Repair parallel cache reuse qualification

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision f95d38e, accept

## Outcome

Concurrent CLI starts reuse a fresh current-epoch cache while its epoch lock is held. Diagnose the current Linux release-qualification extra request and repair its cause without relaxing the no-refetch assertion, freshness, deadlines or cache safety.

- Evidence: full nonpublishing rehearsal 39131401809 on current main `7907ab4b0871922270718db6065a1be7c291431c`, qualification job 117687973670, `make test-python`, failed `tests/test_http_cache_fail_soft.py::test_parallel_current_cache_starts_use_cache_while_epoch_lock_is_held[command0]` at line257: `all parallel starts must reuse the cache`, request-log count `3 == 2`. Summary: 1 failed, 1281 passed, 8 skipped. Current main and its four coverage suites passed on the native builder; this Linux failure needs a cause, not a passing rerun.
- Changes: `tests/test_http_cache_fail_soft.py`, `src/cache/**`, `CHANGELOG.md`. Narrow the production claim after tracing the extra request. Restate the outcome and record the exact current-main focused reproduction before production edits. Preserve the existing production cache/replay test rather than adding a new cache or task mechanism.
- Proof: identify the actual extra provider request and why it bypassed or missed the populated key. Reproduce the relevant cause deterministically, make the real concurrent-process regression pass, preserve genuine fail-open/deadline behavior and ordinary cache-size enforcement, and obtain fresh review. Use occasional Linux build-job reproduction when necessary; routine development gates use the normal builder. Push source once ready, then use the shared landing gate and post-merge main check. Final full nonpublishing rehearsal must pass on the replacement candidate.

## Progress

- 2026-10-18 started

Developer outcome before code: Every concurrent CLI process reads a fresh current-epoch answer despite the held epoch lock, retaining fail-open behavior, deadlines, safety and size enforcement. Initial trace identifies the optional startup-clean path taking an exclusive operation lock before reading its already-current stamp; another client constructor can see that contention and silently bypass the cache. The precise extra provider request still needs confirmation. Candidate minimal production claim is `src/cache/manager.rs`; existing cache/replay and startup-marker tests will establish the cause before edits. Preserve the under-lock recheck and real stale/missing-stamp maintenance. Avoid a new test interception framework.

Deterministic RED before production edits: Linux build job `2208/1792330944436-4015866` ran the existing real 12-process CLI regression offline on `2e078ef21`, adding only request/stderr diagnostics and a temporary syscall-delay library. The library widened only a successful exclusive `.biomcp-operation.lock` interval when the current `.last-startup-clean` marker already existed; it changed no cache bytes and stayed outside the repository. The assertion failed in 1.79s with request count13 versus2. Extra requests were the same `GET /mygene/query` BRAF symbol/alias query. Stderr recorded both `holding current-marker startup maintenance lock` and rejected shared cache leases (`errno=11`). This connects needless current-marker startup maintenance to real concurrent cache bypass and refetch. The scratch library was removed by the job trap. Retain the ordinary existing cache assertions; repair the local stamp fast path and keep the under-lock recheck. No production edit preceded this reproduction.

GREEN: the identical forced-contention Linux diagnostic passed in build job `2208/1792331252385-4021365` (85.660s total, real CLI case0.70s). Native build job `2208/1792331251367-78082` passed in214.561s: all37 cache-manager Rust cases, all20 existing fail-soft/persistent-size Python cases, complete `make lint` and diff check. The CLI was rebuilt after the source edit. The minimal source change skips optional maintenance before acquiring an exclusive lock when its startup stamp is current, retains the inside-lock recheck for missing/stale stamps, and keeps existing cache assertions with added failure diagnostics. No test-only production hook or persistent interposer was added. Ready source is `10aa57d73dec0c147a552f7e4ee8ac1547876bf4`.
