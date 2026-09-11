---
flow: quickfix
priority: 9
deps: []
---

# Make the provider-deadline finalization test clock deterministic

## Goal

`provider_deadline_waits_for_post_publish_fail_closed_finalization` proves its
deadline and finalization contract on every run, under load, without
wall-clock sensitivity.

## Current behavior

The test at `src/sources/tests/provider_network.rs:357-405` gives the request a
10 ms `VariantArticleDeadline` but lets real scheduling, local networking, and
cache setup determine whether the request reaches the safe-return arm before
that deadline. Under load, the request can therefore settle as a deadline error
before the publication notification that the test awaits. The later 20 ms
sleep also uses wall-clock time to establish exhaustion. A separate 40 ms
fixture delay runs only after commit, during finalization; it does not cause the
pre-release race and is unnecessary to this proof. The test failed once on
2026-09-09 in a loaded full-suite rerun and passed in isolation and in the next
full run.

## Required behavior

Convert this test to paused Tokio time (`start_paused = true`). Keep virtual
time from auto-advancing while the local request reaches the publication pause,
using the established driver pattern at the top of the same file. Remove the
real post-write delay from this case. Replace the one-second Tokio timeout with
a select between the publication notification and premature request
settlement. When publication pauses, assert that the deadline has not expired;
then stop and join the driver, advance virtual time past the 10 ms deadline,
assert exhaustion, and release publication.

Preserve the fail-closed proof: an exhausted deadline still waits for cache
publication and returns the sanitized error. The existing request, cache get,
cache put, and sanitized-error assertions remain exact.

## Observable success

The focused test passes repeatedly without consuming its former 40 ms fixture
delay, and the standard gates pass.

## Boundaries

Production deadline and cache code unchanged.

## Complexity

- Contract: 0 — the production contract is unchanged.
- State and timing: 2 — a spawned request and local server coordinate a
  publication notification, virtual-time driver, and deadline boundary.
- Reach: 0 — one existing test changes.
- Proof: 2 — the test uses injected post-write failure plus exact cache
  counters and sanitized-error assertions.
- Cost of error: 0 — no runtime path changes.
- Total: 4.
- Minimum level floor: Level 3 because correctness depends on concurrent task
  and timer coordination.
- Final level: Level 3.
- Reasons: concurrency, paused-clock auto-advance, and failure injection require
  explicit ordering proof even though the edit is test-only.
- Selected model: `gpt-5.6-sol`, medium.
