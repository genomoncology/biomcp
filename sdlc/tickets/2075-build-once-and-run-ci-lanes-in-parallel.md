# 2075 — Build once and run CI lanes in parallel

Status: OPEN.

Milestone: 0.9.3

## Outcome

A green CI run takes about as long as its longest lane, and costs about 30 runner-minutes instead of 70.

## Evidence

- CI runs 8 jobs in 22 to 28 minutes and about 70 runner-minutes; canonical-gates runs lint, test and spec in series; release-panic spends 16.4 minutes on one test; stress-lane rebuilds the archive (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, CI and step 4).

- Starts from: 2070 (one dev build per ticket); `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 4.
- Keeps: every check CI runs today, on main at least.
- Changes: one job builds and uploads the nextest archive and the binary; lint, Rust tests, Python lane and spec lane run in parallel from it; stress-lane reuses the archive; release-panic moves to the release workflow; full-features and Windows run on main only.
- Proof: Actions API wall time and runner-minutes for ten green runs before and after.
- Defers: nothing.
