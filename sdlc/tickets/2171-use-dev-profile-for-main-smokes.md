---
priority: 1
---

# 2171 — Use the development profile for full main checks

Status: OPEN.

Milestone: 0.9.3

## Outcome

Every full main check builds the full feature set with the development profile and exercises its PNG, SVG and terminal output. Optimized builds and their output checks remain required in release qualification and the final release rehearsal.

- Decision: PM approved this scope in coordinator mail titled Decide the optimized build in each main check. Long optimized checks belong at the release candidate.
- Evidence: main check 1792246099764-52005 took 705.265 seconds, including 304 seconds for its optimized stage. Test-only landing check 1792246904631-55484 still spent 117 seconds rebuilding the optimized executable.
- Changes: claims `Makefile` `tests/test_make_entrypoints.py` `docs/reference/release-process.md` `AGENTS.md` `CHANGELOG.md`; extend claims only for the existing stage implementation or its existing contracts. Keep full-feature build, all-target Clippy, AlphaGenome cases and output checks required. Keep optimized build and output checks required in release qualification and final rehearsal.
- Proof: first reproduce current profile selection with the existing command-level harness on current main. Show the same contract failing before and passing afterward; prove main selects the development executable while release qualification still selects and exercises the optimized executable. Run affected checks through pm job, then the normal landing gate and full main check. No substitute gate and no skipped stages.
- Before coding: restate the outcome, reproduce current behavior and name the files and failing outside-in check in the ticket through the coordinator.
