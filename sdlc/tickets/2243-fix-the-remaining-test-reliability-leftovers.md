---
priority: 2
---

# 2243 — Fix the remaining test reliability leftovers

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `CHANGELOG.md` `Makefile` `docs/reference/source-licensing.md` `docs/reference/sources.json` `sdlc/issues/2026-09-27-lifecycle-tests-fail-in-copied-workspaces.md` `sdlc/issues/2026-09-28-review-follow-ups-after-1255-through-1261.md` `sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md` `sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md` `sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md` `sdlc/tickets/2243-fix-the-remaining-test-reliability-leftovers.md` `src/cache/manager.rs` `src/cli/discover.rs` `src/cli/outcome/probe_tests.rs` `src/cli/tests/facade/cache.rs` `src/cli/update/tests.rs` `src/cli/worker.rs` `src/sources/gencc/tests.rs` `src/test_support.rs` `src/utils/sync.rs` `tests/test_routine_cargo_feature_contract.py` .

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

## Coordinator development brief

Repair the inventoried eleven unbounded test joins under src using a small existing-watchdog-based bounded join that preserves panic results and named diagnostics. Leave the production outcome join and already bounded article receive unchanged. Sync Python before both spec targets, remove only the process-wide counter assertion while retaining the armed-window probe, and apply the explicitly requested source note. The current ruling supersedes the old issue disposition retaining both counter assertions. Do not expand into integration/Python wait inventories. Return issue-closure text for coordinator rather than editing additional unclaimed issue files. Reproduce Makefile ordering with a red regression, then focused affected tests. The later2244 lane must account for this GenCC test-file overlap.

All builds and tests run through the configured build job. Stage private test/source patches on a published base without pushing WIP; retain patch and per-file hashes with raw evidence, then verify the final published source matches. Push source once ready for review. The developer may resolve the named issue findings as required by the outcome, but does not edit ticket status or send mail.

## Focused implementation proof

Accepted source6a8af28129a096b14f304ae28b498c67a3b52c51 changes fourteen product files plus this record. The two Makefile ordering cases failed first; focused green checks passed 30 Python cases with one intentionally excluded repository audit/landing combination, formatting, Ruff, ratchet, all-target Clippy and affected Rust cases. A direct Darwin invocation initially omitted the routine real TMPDIR and failed three GenCC Store::open cases; the corrected published-source job1792421121441-71898 passed all three. This is not evidence of the separate one-CPU bug. Publication job1792421079065-65198 matched fifteen hashes and patchSHA256 1c644352fbe19f80261de2ee5915072dc42bea7bbd50c02a12c83a5a58ca99b7.

Fresh source reviewer accepted6a8af28129a096b14f304ae28b498c67a3b52c51. Reviewer probe1792421584453-32575 passed helper success value, panic payload, timeout error and cleanup during unwinding. No source edits followed that acceptance. Raw developer and reviewer evidence is retained in experiments/781-biomcp-2243-test-reliability. The source registry and generated licensing reference now carry the requested DepMap clinical-use note. This changes the gate definition and lands separately.

Keep: experiments/781-biomcp-2243-test-reliability — Retain cited raw proof until release evidence is explicitly released.
