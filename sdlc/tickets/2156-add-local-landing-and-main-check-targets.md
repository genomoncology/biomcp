# 2156 — Gate each landing locally and check main before the next

Status: OPEN.

Milestone: 0.9.3

## Outcome

The existing landing command checks the reviewed merge candidate through the build job, and the coordinator runs the complete standard and shipped-feature checks on main before another ordinary landing.

Depends on: 2155

- Starts from: `Makefile` routine and full-feature targets, `bin/lint`, the top-level gate configuration in `sdlc/pm.json`, `AGENTS.md`, and existing gate contracts.
- Keeps: routine gates share the no-default-feature build graph; executable specifications and strict docs remain covered; long, load, coverage and release-only packaging checks stay outside per-landing targets; the existing pm merged-candidate gate and pass reuse remain authoritative.
- Changes: claims `Makefile` `bin/lint` `sdlc/pm.json` `AGENTS.md` `tests/test_routine_cargo_feature_contract.py` `tests/test_offline_gate_contract.py` `CHANGELOG.md`.
- Proof: establish current main lacks the named landing/main targets and the configured tickets gate; extend the existing outside-facing command contracts so they fail before implementation. `make landing-gate` runs lint, routine tests and routine specifications; `make main-gate` runs those standard checks and the shipped-feature checks; neither selects release-only tests. A deliberately lint-invalid test target must fail the routine lint command. Exercise the new pm gate through one real landing and run the main target on its actual pushed merge revision.
- Defers: automatic repository-wide postmerge holds and landing locks belong to the existing pm follow-up. Shared role guidance is routed through the owning PM. Do not add a new evidence or queue framework.

PM decision: 2026-10-17. Ian approved the existing tickets gate as the premerge check and a single coordinator waiting for the postmerge result. Declare top-level `ticketsGate` as `make landing-gate`; remove obsolete GitHub merge conditions and deprecated configuration keys that prevent the configured gate from loading. Lint includes test targets. Public repository text describes build jobs without private machine names. The coordinator records a failed or absent current-main result as a hold; only a reviewed repair or backout may land until main passes. On resume inspect current main's exact-revision job evidence before another landing. The release workflow and long-suite policy are separate work.

## Reproduction

PM decision needed: route the approved process into the shared coordinator, developer and reviewer skills. Replace GitHub CI landing requirements with the existing build-job quick gate, sole lander, postmerge complete main check and failure hold. Keep long/load checks at the candidate or affected behavior and routine elapsed-time warnings with load. Same-team mail attempts were rejected by the installed tool, so this request follows the PM's temporary ticket-line instruction.

On main `67c42424e`, job `1792238941539-82305` ran `pm job run 2156 67c42424e --disk 1 --memory 1 -- make -n landing-gate main-gate` and exited 2 with `No rule to make target landing-gate`. The top-level gate configuration is also absent. No intervening main repair changes either target or configuration. The inherited lint gap is evidenced by the test-code warnings caught only by the all-target full-feature command; include test targets in the routine lint entry point.
