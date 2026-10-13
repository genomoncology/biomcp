# 2080 — Make the spec pages fast

Status: OPEN.

Milestone: 0.9.3

## Outcome

`make spec` takes a few minutes, with no retry back-off sleeps, no 40-line blocks, and no fixture scripts.

## Evidence

- `make spec` takes about 11 minutes and waits more than it computes: simulated provider failures sleep through retry back-off, and the section-outcomes page alone takes about two minutes (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 8).
- 22 blocks exceed 40 lines; the runner is 25 KB of bash with per-page fixture scripts.

- Starts from: 2079 (replay server and golden suite).
- Keeps: the pages as reader-facing specifications with `like` and `expect=` assertions.
- Changes: a test setting turns retry back-off to zero and the article deadline fixtures use an injected clock; move JSON assertions and request-log arithmetic into golden cases; cut every block over 40 lines; replace the fixture scripts with the replay server; delete the lifecycle and parallel-isolation tests with them.
- Proof: `make spec` wall time before and after; runner line count; no block over 40 lines.
- Defers: nothing.
