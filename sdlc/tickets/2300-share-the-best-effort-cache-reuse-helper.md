---
priority: 2
---

# 2300 — Share the best-effort cache reuse helper

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every test that counts cache writes or provider requests after concurrent runs asserts the contract through one shared helper: reuse of the entries actually admitted, never that every concurrent write landed. `admitted_provider_keys` moves from tests/test_http_cache_fail_soft.py:32 into shared test support, and one sweep routes every concurrent count assertion through it.

- Proof: A grep finds no concurrent request-count assertion outside the helper; a controlled broken reuse still fails; a skipped write under contention passes.

Changes: `tests/golden/cache_reuse.py`, `tests/test_http_cache_fail_soft.py`, `tests/test_cache_sqlite.py`, `CHANGELOG.md`, `sdlc/records/2300-shared-cache-reuse-contract.md`.

## Evidence

2225, 2275, 2283 and 2285 each failed a full run on a skipped best-effort write while every answer was correct. The 2283 audit missed 2285's internal fan-out because the sweep was a manual read. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
