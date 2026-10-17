---
priority: 1
---

# 2171 — Use the development profile for full main checks

Status: OPEN.

Milestone: 0.9.3

Reviews: revision b72ad5d, accept

Reviews: revision d19fa79, accept

Reviews: revision 4aa761b, accept

Reviews: revision 38fe6fd, accept

Reviews: revision cec8432, accept

## Outcome

Every full main check builds the full feature set with the development profile and exercises its PNG, SVG and terminal output. Optimized builds and their output checks remain required in release qualification and the final release rehearsal.

- Decision: PM approved this scope in coordinator mail titled Decide the optimized build in each main check. Long optimized checks belong at the release candidate.
- Evidence: main check 1792246099764-52005 took 705.265 seconds, including 304 seconds for its optimized stage. Test-only landing check 1792246904631-55484 still spent 117 seconds rebuilding the optimized executable.
- Changes: claims `tests/rmcp_client_contract.rs` `Makefile` `tests/test_routine_cargo_feature_contract.py` `tests/test_upstream_planning_analysis_docs.py` `docs/reference/release-process.md` `AGENTS.md` `CHANGELOG.md`; extend claims only for the existing stage implementation or its existing contracts. Keep full-feature build, all-target Clippy, AlphaGenome cases and output checks required. Keep optimized build and output checks required in release qualification and final rehearsal.
- Proof: first reproduce current profile selection with the existing command-level harness on current main. Show the same contract failing before and passing afterward; prove main selects the development executable while release qualification still selects and exercises the optimized executable. Run affected checks through pm job, then the normal landing gate and full main check. No substitute gate and no skipped stages.
- Before coding: restate the outcome, reproduce current behavior and name the files and failing outside-in check in the ticket through the coordinator.

## Pre-code outcome and proof plan

The developer will make main build and smoke the full feature set with the development executable; release qualification and rehearsal will still build and smoke the optimized executable. Inspection at 43eb1cdff reproduces the current path: main-gate calls full-feature-check, which builds with --release and smokes target/release/biomcp. This is an inspection result, not an executed failing test. The existing command harness is tests/test_routine_cargo_feature_contract.py; the initially named test_make_entrypoints.py does not exist. Extend the existing harness to execute real Make recipes with controlled cargo and smoke leaves, expecting development/debug for main and release/release for qualification. Required red and green commands remain blocked by build-host login refusal. No executed proof is claimed.

## Progress

- 2026-10-17 started
- 2026-10-17 next: Source review prepared at 54bf2c3cd after integrating accepted batch 2159, 2149, 2168, 2128 and 2122 with current main. Reviewer fixed audit-stage harness integration without changing the required audit. Required red at ef4bb2526 and green at final candidate remain unexecuted because build-host authentication is refused. No final accept or landing is claimed.
- 2026-10-17 next: Review accepted b72ad5dda. M5 red 1792249759100-57543 proves old main chooses optimized profile; green 1792249770832-17639 passes 39 affected contracts with one intentional skip. That job later hit a lint-driver path mistake; corrected lint 1792249787439-89343 passes without repeating tests. Batch landing gate is running for 2159, 2149, 2168, 2128, 2122 and 2171; required current-main full check runs separately before landing.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared landing gate: job 1792250897357-93113, candidate ea4161cc814ad034634ffd249e67b475e77e38dc. Required preceding main check 1792249698624-96397 passed. Gate is running; each ticket keeps its individual review and landing commit.
- 2026-10-17 next: Batch members: 2159, 2149, 2168, 2128, 2122 and 2171. Shared configured landing gate PASSED: job 1792251400030-83132, candidate c264da2a525c2c6e5a424693d55dfab0f59fadf4, 240.3 seconds. This replaces the earlier failed batch attempts. Preceding full main check 1792249698624-96397 passed. Land each reviewed member through pm, then run one full check on final batch main.

## Landing gate follow-up

The identical-code landing repeat failed the existing raw/typed adverse-event MCP contract at its fourth sequential provider request after the preceding configured batch gate passed. The coordinator assigned one focused reviewer to the carrier, as instructed for unrelated gate failures. The existing assertions omit the MCP error text. Add assertion diagnostics in tests/rmcp_client_contract.rs, preserve the product assertions and use one targeted build job to identify the error. No runtime cache change or clinical decision is authorized by this follow-up; report a cause belonging to the queued cache fixes instead of duplicating their work.
