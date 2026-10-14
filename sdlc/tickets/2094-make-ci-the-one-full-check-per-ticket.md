# 2094 — Make CI the one full check per ticket

Status: OPEN.

Milestone: 0.9.3

Depends on: 2075
Depends on: 2071
Depends on: 2076
Depends on: 2080

## Outcome

Ticket-branch CI has a median wall time of about 10 minutes over ten full runs, `make test` takes about 3 minutes on the build host, and `AGENTS.md` tells a developer to run only the tests for what changed and leave the one full check to CI.

## Evidence

- Ian ruled on 2026-10-14 that 25 minutes is too long even for CI, that ticket-branch CI runs the routine suite only, and that once it is at about 10 minutes CI is a ticket's one full check ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)). Today a developer runs `make lint`, `make test` and `make spec` in series before landing (196 s, 640 to 646 s and 657 to 668 s in `sdlc/records/2069-0-9-3-baseline.md`), and CI runs them again.
- 2075 runs the lanes in parallel off one build and moves the heavy jobs to `main`; 2071 and 2076 shorten the Python lane; 2080 shortens the spec lane. This ticket measures the result and closes any gap left.

- Starts from: CI and `make test` as they stand after 2075, 2071, 2076 and 2080; `AGENTS.md`, section Working.
- Keeps: every routine check on every ticket push; the heavy jobs on `main` and the release workflow; `make test`, `make lint` and `make spec` as the commands a developer can still run.
- Changes: claims `AGENTS.md` `CHANGELOG.md`, plus any lane file a remaining gap needs, named on this ticket before the first push.
  - `AGENTS.md`, Working: while working, run only the tests for what changed; push, and CI is the ticket's one full check; run `make full-feature-check` before landing when a change touches code only the all-features build compiles.
  - If the median or `make test` misses its target, cut the lane that sets the floor: delete tests that are not one of the four kinds, or move a release-only check to the release workflow.
  - One Internal changelog bullet.
- Proof: wall time from the Actions API for ten full ticket-branch runs after landing; `tools/measure-0-9-3-baseline <clone> --only make-test` on the build host; the `AGENTS.md` diff.
- Defers: nothing.
