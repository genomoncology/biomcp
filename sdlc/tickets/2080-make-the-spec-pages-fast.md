# 2080 — Make the spec pages fast

Status: OPEN.

Milestone: 0.9.3

## Outcome

`make spec` takes about 3 minutes on the build host, with no retry back-off sleeps, no 40-line blocks, and no fixture scripts.

## Evidence

- `make spec` takes about 11 minutes and waits more than it computes: simulated provider failures sleep through retry back-off, and the section-outcomes page alone takes about two minutes (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 8).
- 22 blocks exceed 40 lines; the runner is 25 KB of bash with per-page fixture scripts.

- Starts from: 2079's first batch (replay server and golden suite); it starts once that batch lands, not when all of 2079 does.
- Keeps: the pages as reader-facing specifications with `like` and `expect=` assertions.
- Changes: claims `scripts/run-specs.sh` `scripts/prepare-spec-artifacts.py` `spec/fixtures/**` `CHANGELOG.md`, and each spec page it edits, named on this ticket before the first push. A test setting turns retry back-off to zero and the article deadline fixtures use an injected clock; move JSON assertions and request-log arithmetic into golden cases; cut every block over 40 lines; replace the fixture scripts with the replay server; delete the lifecycle and parallel-isolation tests with them.
- Proof: `make spec` wall time on the build host before and after, against the 2069 baseline's 657 to 668 s and the decision's 3-minute target ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)); runner line count; no block over 40 lines. A page that cannot reach the target without losing a reader-facing claim moves its JSON checks into golden cases or goes.
- Defers: nothing.
