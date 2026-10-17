---
priority: 3
---

# 2094 — Run the local test lanes in parallel

Status: COMPLETE.

Milestone: 0.9.3

Depends on: 2081
Depends on: 2156

Reviews: revision 671f8ea, accept

Reviews: revision ec7d139, accept

## Outcome

`make test` prepares shared artifacts once, runs the Rust and Python lanes concurrently with readable separate output, and fails when either lane fails while preserving every retained check.

- Starts from: the existing implementation branch and the `prepare-test`, `test`, `test-rust` and `test-python` targets in `Makefile`.
- Keeps: one Rust build at a time, isolated test temporary directories, retained audit coverage, compiler caches and supported build-host portability.
- Changes: claims `tests/test_python_contract_parallelism.py` `tests/test_upstream_planning_analysis_docs.py` `Makefile` `AGENTS.md` `tests/test_routine_cargo_feature_contract.py` `CHANGELOG.md`. Retain useful local preparation and parallel execution work from the existing branch. Use portable CPU discovery and measured worker settings. Remove hosted setup-cache work and instructions assigning qualification to automatic CI.
- Proof: through the build job, run `make test` on the pushed revision; prove preparation occurs once and either lane's failure propagates. Compare with current main on the same host and settings. Report observed time and savings without a hosted-CI target.
- Defers: local audit scheduling to 2122; slow-test warnings to 2095; final comparison to 2082.

PM decision: 2026-10-17. Ian replaced automatic ticket CI with local development, merged-candidate routine checks and postmerge standard/shipped-feature checks. The prior accepted review covers the former contract; review the revised branch after rebasing. Preserve any historical measurements as historical evidence, not current acceptance proof.

PM decision: 2026-10-17. Back in 0.9.3. Faster tests and builds are a 0.9.3 goal, and this ticket carries it. Rebase onto main before resuming; earlier reviews cover the old base only.

## Rebased implementation and proof

The local runner prepares once, runs retained Rust and Python lanes concurrently with separately labeled buffered output, waits for both lanes, propagates either failure and removes temporary logs. Rust workers use portable discovery capped at four; Python retains four workers. Focused Make/docs contracts passed 36 cases with Ruff. Job `1792244325098-99402` reused one unchanged prepared archive without rebuilding: serial lane execution took 223.631 seconds; concurrent execution took 137.362 seconds, an observed 38.6 percent reduction under shared load. The serial Rust run passed 4336 tests and hit the existing cancellation fixture defect; the concurrent Rust run passed all 4337. Python passed 1158 tests with eight skipped and the known moved-ticket changelog failure. Both timing samples skipped the subsequent strict documentation build after that failure, so this comparison measures test lanes and does not establish a green complete gate. Clock/socket repair 2167 and changelog repair 2166 cover the unrelated failures. Final whole-gate and actual-main checks remain required.
