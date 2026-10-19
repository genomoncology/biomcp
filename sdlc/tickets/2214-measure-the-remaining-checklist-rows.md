---
priority: 3
---

# 2214 — Measure the remaining checklist rows

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 6f362b3, accept

## Outcome

The release measurement covers every row the 0.9.3 checklist named, so the next release compares like with like.

- Evidence: the 0.9.3 baseline record lacks these checklist rows: build folder size of a working checkout, crates compiled for the Linux and Windows release targets, Python test function count and the share that tests BioMCP's own tooling, the edit-then-run-one-test-group loop, release workflow run time, archive download size, and the number of full rehearsals a release needed.
- Changes: `tools/measure-0-9-3-baseline` `tests/test_measurement_summary_contract.py` `sdlc/records/2069-0-9-3-baseline.md` and this ticket. These are the exact reviewed implementation paths; the coverage contract is unchanged.
- Proof: each row reported for v0.9.3 and the 0.9.4 candidate on one build machine, with the method stated; rows that cannot be measured say why.

- Changes: `tools/measure-0-9-3-baseline` `tests/test_measurement_coverage_contract.py` `tests/test_measurement_summary_contract.py` and the baseline record. Inventory the exact record path and narrow tests before editing. Reuse retained release workflow and archive evidence; mark unavailable historical rows honestly. Measure frozen revisions; do not call a moving development tip the final release candidate.

## Development brief

Restated outcome: extend the existing measurement tool so every named checklist row has a value or explicit limitation, with honest immutable product identities.

Reproduction: current main `08b74a340658f2c30e158fd00459330baebacd74`, job `2214/1792409864323-63343`, static Python inspection of the helper and record (complete command retained in the invocation), exit 0 in 44 ms. Output: `MISSING REQUESTED SECTIONS: working-checkout, release-target-counts, python-test-functions, edit-test-group, release-history`. Existing `RESULT crates_compiled_{label}`, `build_folder_bytes_after_make_test`, `python_tests_collected`, and `build_folder_bytes_after_release_build` do not establish those requested rows.

Exact planned files: `tools/measure-0-9-3-baseline`, `tests/test_measurement_summary_contract.py`, `sdlc/records/2069-0-9-3-baseline.md`. Leave the coverage contract unchanged unless an actual interaction requires it.

Outside-in red case: invoke the real helper against the existing temporary fixture checkout/fake external tools and assert requested row values or explicit unavailability, separate Python function and parametrized case counts, platform identities, working-folder bytes and the actual edit-then-test command. Current helper lacks those outputs. Parse Python AST for function definitions; retain the tooling classification inventory. Do not relabel resolved Cargo graphs as compiled crates. Reuse retained target logs and 2215 release evidence rather than dispatch new workflows or platform builds. Historical checkout bytes may require an explicit recreated-state limitation.

The old baseline record primarily compares v0.9.2 and earlier development code. New rows compare published v0.9.3 `5641de7d7b79b53872f8599dbb4dba22e611dcaf` with frozen development snapshot `08b74a340658f2c30e158fd00459330baebacd74` on one configured build host. Never call that snapshot the final release candidate. Retained 2215 record already supplies publishing duration, all five archive sizes and eight full-mode attempts (three failed, three cancelled, two successful).

## Progress

- 2026-10-19 started
