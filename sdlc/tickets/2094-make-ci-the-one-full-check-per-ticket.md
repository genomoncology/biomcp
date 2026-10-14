# 2094 — Make CI the one full check per ticket

Status: OPEN.

Milestone: 0.9.3

Depends on: 2075
Depends on: 2071
Depends on: 2076
Depends on: 2080
Depends on: 2081

## Outcome

Ticket-branch CI has a median wall time of 11 minutes or less over the branch's own full runs, `make test` takes 200 s or less on the build host, and `AGENTS.md` tells a developer to run only the tests for what changed and leave the one full check to CI.

## Evidence

- Ian ruled on 2026-10-14 that 25 minutes is too long even for CI, that ticket-branch CI runs the routine suite only, and that once it is at about 10 minutes CI is a ticket's one full check ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)). Today a developer runs `make lint`, `make test` and `make spec` in series before landing (196 s, 640 to 646 s and 657 to 668 s in `sdlc/records/2069-0-9-3-baseline.md`), and CI runs them again.
- 2075 runs the lanes in parallel off one build and moves the heavy jobs to `main`; 2071 and 2076 shorten the Python lane; 2080 shortens the spec lane; 2081 deletes the tests that are not one of the four kinds.
- The work above does not reach the targets alone. In run 38545958060, canonical-gates spent 2.6 minutes on setup before any gate ran, 1.8 of them installing the gate tools; after 2075 the build job and every lane job each pay that setup, so the build job plus a lane is about 6 to 7 minutes plus the lane itself. `make test` after 2071 and 2076 still runs about 5.5 minutes on the build host by the Python lane's per-file sums (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3), because the Rust and Python lanes run one after the other.

- Starts from: CI and `make test` as they stand after 2075, 2071, 2076, 2080 and 2081; the composite setup action 2075 adds under `.github/actions/`; `AGENTS.md`, section Working.
- Keeps: every routine check on every ticket push; the heavy jobs on `main` and the release workflow; `make test`, `make lint` and `make spec` as the commands a developer can still run.
- Changes: claims `.github/workflows/ci.yml` `.github/actions/**` `Makefile` `tests/test_ci_workflow_contract.py` `AGENTS.md` `CHANGELOG.md`.
  - Cache the installed gate tools (nextest, mustmatch, uv's environment, the offline sandbox packages) in the composite setup action, keyed on their pinned versions, so a lane job's setup takes under a minute.
  - The build job restores the Rust build cache and builds only what the lanes download.
  - `make test` runs `test-rust` and `test-python` at the same time, each writing its own output, and fails when either fails.
  - `AGENTS.md`, Working: while working, run only the tests for what changed; push, and CI is the ticket's one full check; run `make full-feature-check` before landing when a change touches code only the all-features build compiles.
  - If the median or `make test` still misses its target, cut the lane that sets the floor: delete tests that are not one of the four kinds, or move a release-only check to the release workflow.
  - One Internal changelog bullet.
- Proof: before landing, the branch's own CI run plus five reruns (`gh run rerun`), with the median wall time of the six from the Actions API at 11 minutes or less; five reruns, not ten, because reruns of one run go one after another and each costs wall time; `tools/measure-0-9-3-baseline <clone> --only make-test` on the build host at 200 s or less, against the 2069 baseline's 640 to 646 s; the `AGENTS.md` diff. Both timings fail on main today.
- Defers: nothing.
