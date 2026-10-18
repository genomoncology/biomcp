---
priority: 1
---

# 2206 — Bound cache eviction by the request deadline

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 662d893, accept

## Outcome

A cache write cannot make a variant or article request wait beyond its deadline for another operation to release a cache lease. Keep committed entries valid and preserve ordinary short-run cache-size enforcement.

- Evidence: a follow-up review of landed 2195 found that the new eviction await runs inside safe-return deadline masking. Spawned eviction tasks inherit no request deadline, and maintenance waits indefinitely for another shared lease. The outer request guard also drains after publication is armed, so it cannot bound this wait.
- Starts from: current main, including the persistent counter and short-process eviction fix from 2195.
- Keeps: cache security, atomic accounting, completed-write integrity, ordinary eviction waiting, and provider fallback.
- Changes: `src/cache/manager.rs`, `src/cache/manager/**`, `tests/test_http_cache_fail_soft.py`, `CHANGELOG.md`. Claim only files the minimal deadline repair needs; avoid a new shutdown or task-management mechanism.
- Proof: hold another production cache lease; write over the size budget inside a real request deadline and publication scope; observe the pre-fix wait beyond the deadline, release the held lease, and drain before asserting failure. The repaired request must finish within its deadline while the existing ordinary eviction-wait and repeated-short-process size cases still pass. Prefer the existing CLI fixture when it can exercise the same path without a new harness.

- RED: M5 job `2206/1792325854229-80648`, current main `31ec1c22b` with only the regression test added: `cargo nextest run --no-default-features -E 'test(put_does_not_wait_past_request_deadline_for_eviction)'`. Failed as expected: 1 failed, 4406 skipped, assertion `optional eviction waited beyond the request deadline`. The held lease was released and the operation drained before the assertion. Production code was unchanged.

- GREEN: M5 job `2206/1792326216881-79245` passed in 252.936s on the final three-file source patch. Three focused Rust cases passed: `put_does_not_wait_past_request_deadline_for_eviction`, `eviction_waits_for_an_active_cache_operation`, and `put_deadline_transition_is_exactly_at_writer_commit`. The existing `tests/test_cache_persistent_size.py` and `tests/test_http_cache_fail_soft.py` passed all 20 cases in 30.35s; `make lint` and `git diff --check` passed. Fresh reviewer `/root/review_cache_eviction_deadline` accepted `662d89369a14995b4d5af0922127b5ddd95f8678` after reading the diff, callers, masking and cleanup checkpoints, reusing this evidence.

## Progress

- 2026-10-18 started
