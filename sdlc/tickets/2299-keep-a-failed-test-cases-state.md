---
priority: 2
---

# 2299 — Keep a failed test case's state

Status: OPEN.

Milestone: 0.9.4

## Outcome

When a replay-backed test fails, its case directory, cache database and replay logs survive for diagnosis. tests/golden/replay_env.py borrows a pytest-owned case directory, following the 2285 pattern, so teardown no longer deletes the evidence.

- Proof: A deliberately failing replay test leaves its database and logs under the pytest case directory, and a passing test cleans up as before.

## Evidence

Four lanes (2275, 2283, 2285, 2298) ended with the original cause unknown because fixture teardown deleted the failed case's database before anyone could read it. See the root cause record `sdlc/records/2299-root-cause-of-test-only-check-failures.md`.
