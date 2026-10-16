---
priority: 3
---

# 2094 — Run the local test lanes in parallel

Status: OPEN.

Milestone: 0.9.3

Depends on: 2081
Depends on: 2156

Reviews: revision f013bfc, accept

## Outcome

`make test` prepares shared artifacts once, runs the Rust and Python lanes concurrently with readable separate output, and fails when either lane fails while preserving every retained check.

- Starts from: the existing implementation branch and the `prepare-test`, `test`, `test-rust` and `test-python` targets in `Makefile`.
- Keeps: one Rust build at a time, isolated test temporary directories, retained audit coverage, compiler caches and supported build-host portability.
- Changes: claims `Makefile` `AGENTS.md` `tests/test_routine_cargo_feature_contract.py` `CHANGELOG.md`. Retain useful local preparation and parallel execution work from the existing branch. Use portable CPU discovery and measured worker settings. Remove hosted setup-cache work and instructions assigning qualification to automatic CI.
- Proof: through the build job, run `make test` on the pushed revision; prove preparation occurs once and either lane's failure propagates. Compare with current main on the same host and settings. Report observed time and savings without a hosted-CI target.
- Defers: local audit scheduling to 2122; slow-test warnings to 2095; final comparison to 2082.

PM decision: 2026-10-17. Ian replaced automatic ticket CI with local development, merged-candidate routine checks and postmerge standard/shipped-feature checks. The prior accepted review covers the former contract; review the revised branch after rebasing. Preserve any historical measurements as historical evidence, not current acceptance proof.

PM decision: 2026-10-17. Back in 0.9.3. Faster tests and builds are a 0.9.3 goal, and this ticket carries it. Rebase onto main before resuming; earlier reviews cover the old base only.
