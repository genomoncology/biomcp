# 2081 — Delete unit tests the outside-in suite covers

Status: OPEN.

Milestone: 0.9.3

## Outcome

Unit tests remain only for parser edge tables, security boundaries and regressions, combined coverage stays within a few points of 84%, and no test waits on the wall clock.

## Evidence

- Outside-in tests cover 65% of production lines, unit tests 74%, both 84%; 74% of unit-covered lines are also reached from outside. 75 files are covered 90% or more from outside; 34 files are never reached from outside (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, Coverage, step 7).
- 81 tests take a second or more, 399 s of 528 s of Rust test time.
- 63 test-only items (about 450 lines) and 58 unread fields exist for construction tests (`sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md`, Dead code).

- Starts from: 2079 (golden suite) and the coverage method in `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`.
- Keeps: parser edge tables, security boundary tests, regression tests; a golden case before any sole test of a behavior is deleted.
- Changes: delete unit tests file by file, starting with the 75 files; add outside-in cases for the 34 untouched files before deleting there; give the slow tests an injected clock or a shorter deadline; delete the test-only items and unread fields as their tests go; land in small batches.
- Proof: combined coverage within a few points of 84%; Rust test count, lib test compile time and run time fall; the dead-code scan in `sdlc/designs/2026-10-13-0-9-3-dependencies-and-dead-code.md` reports no hand-written spans.
- Defers: nothing.
