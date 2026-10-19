---
priority: 2
---

# 2233 — Run the Linux offline suites from a fresh clone

Status: OPEN.

Milestone: 0.9.4

## Outcome

- Changes: `sdlc/records/2233-linux-offline-suites.md` and this ticket during the study phase; no product edits are authorized before the exact source claim is declared.

The full Rust and Python suites pass on the Linux test machine from a fresh clone of current main with the network off, before any 0.9.4 release rehearsal, and each gap found has its own ticket.

## Evidence

The plan requires this check before any rehearsal. Clean-machine setup gaps forced repeated 0.9.3 rehearsals. Running it now, while 2134 and the replay work land, finds gaps before the release instead of during it.

## Keeps

Change no product code in this ticket. Record each failure with its cause and file a fix ticket for it. Use only the Linux test machine and remove the clone afterwards.

## Proof

One record lists the commit, the commands, the pass and fail counts and each failure's cause. Rerun once on the final release candidate after 2134 and the replay work land.

## Development brief

Run the declared full offline Rust and Python suites on the configured Linux test machine through a build job, using a fresh clone of current main 096c11fc416ddc18ee3b05814a6d0b9fede63648. Separate dependency preparation from actual network-isolated execution, prove kernel isolation and record exact commands, graphs, counts and exclusions. Collect failures without fixing product code. Report each distinct gap and its cause to the coordinator, who files its ticket. Remove only owned clone/build scratch after preserving evidence. This is the initial check; final candidate rerun remains required after cache and replay land. PM explicitly authorized this occasional Linux qualification.
