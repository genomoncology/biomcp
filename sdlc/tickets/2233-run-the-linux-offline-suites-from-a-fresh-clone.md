---
priority: 2
---

# 2233 — Run the Linux offline suites from a fresh clone

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `sdlc/records/2233-linux-offline-suites.md`, `CHANGELOG.md`.

The full Rust and Python suites pass on the Linux test machine from a fresh clone of current main with the network off, before any 0.9.4 release rehearsal, and each gap found has its own ticket.

## Evidence

The plan requires this check before any rehearsal. Clean-machine setup gaps forced repeated 0.9.3 rehearsals. Running it now, while 2134 and the replay work land, finds gaps before the release instead of during it.

## Keeps

Change no product code in this ticket. Record each failure with its cause and file a fix ticket for it. Use only the Linux test machine and remove the clone afterwards.

## Proof

One record lists the commit, the commands, the pass and fail counts and each failure's cause. Rerun once on the final release candidate after 2134 and the replay work land.

## Development brief

Run the declared full offline Rust and Python suites on the configured Linux test machine through a build job, using a fresh clone of current main 096c11fc416ddc18ee3b05814a6d0b9fede63648. Separate dependency preparation from actual network-isolated execution, prove kernel isolation and record exact commands, graphs, counts and exclusions. Collect failures without fixing product code. Report each distinct gap and its cause to the coordinator, who files its ticket. Remove only owned clone/build scratch after preserving evidence. This is the initial check; final candidate rerun remains required after cache and replay land. PM explicitly authorized this occasional Linux qualification.

## Progress

- 2026-10-19 started
- 2026-10-19 next: Initial fresh-clone offline qualification passed on096c11fc416ddc18ee3b05814a6d0b9fede63648 with no demonstrated gaps. Preserve evidence; rerun once on the final candidate after2134 and replay work land.
- 2026-10-19 next: Final frozen-source Linux offline qualification is running at 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 in job 1792451222629-265544; retain initial and final evidence separately.
- 2026-10-19 next: Final Linux job1792451222629-265544 failed only concurrent CLI/MCP cache-entry proof on51f5e03. Rust4279passed; Python1587passed/1failed/5skipped; docs/full-feature notrun. Evidence170/final retained and scratch removed. Repair2283 owns diagnosis; rerun final qualification after reviewed repair.

## Final source assignment

Run the final fresh-clone offline qualification on frozen pushed source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 after the replay fixture batch. Preserve the earlier initial snapshot as historical proof; final evidence belongs in a separate subdirectory of the existing qualifying-host experiment. Reuse the established preparation, kernel isolation and command method, with two Rust workers and serial Python. No product changes, new harness or provider calls. If a gap appears, stop with its exact output and separate repair owner. Only the owned fresh clone and build scratch may be removed after evidence retention.

Keep: experiments/170-biomcp-2233-linux-offline-evidence — retain initial and final offline qualification logs, commands, source identities, isolation proof and artifacts.
