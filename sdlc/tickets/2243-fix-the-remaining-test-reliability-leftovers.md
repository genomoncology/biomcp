---
priority: 2
---

# 2243 — Fix the remaining test reliability leftovers

Status: OPEN.

Milestone: 0.9.4

## Outcome

Known test-reliability leftovers are gone: spec targets sync Python, no test asserts on a process-wide counter, and no test waits on a channel or thread without a bound.

## Evidence

1. Finding 17 of `sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`: `make spec-pr` and `make spec-contracts` skip `sync-python-dev` (`Makefile` near lines 179-183; `make spec` has it).
2. `sdlc/issues/2026-09-28-review-follow-ups-after-1255-through-1261.md` and `...-after-1263-and-1264.md`: `the_mcp_path_builds_no_one_shot_runtime` asserts a process-wide counter (`src/cli/outcome/probe_tests.rs` near lines 68 and 78-81) and can flake under plain `cargo test`. Keep the armed-window probe counter and drop the global assertion.
3. `sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md`: about ten unbounded `recv` and `.join()` waits in test code under `src/` were left for a later ticket. Convert them to bounded waits as 2030 did.
4. Small leftovers: add the "not for clinical use" note to the DepMap row in `docs/reference/sources.json`, and mark `sdlc/issues/2026-09-27-lifecycle-tests-fail-in-copied-workspaces.md` resolved, since its tests were deleted in e7e45eba5.

## Keeps

Test promises are unchanged. No new test framework.

## Proof

Each item has its change and the affected tests pass on the build machine. Mark each issue resolved.
