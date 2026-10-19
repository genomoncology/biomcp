---
priority: 2
---

# 2233 — Run the Linux offline suites from a fresh clone

Status: OPEN.

Milestone: 0.9.4

## Outcome

The full Rust and Python suites pass on the Linux test machine from a fresh clone of current main with the network off, before any 0.9.4 release rehearsal, and each gap found has its own ticket.

## Evidence

The plan requires this check before any rehearsal. Clean-machine setup gaps forced repeated 0.9.3 rehearsals. Running it now, while 2134 and the replay work land, finds gaps before the release instead of during it.

## Keeps

Change no product code in this ticket. Record each failure with its cause and file a fix ticket for it. Use only the Linux test machine and remove the clone afterwards.

## Proof

One record lists the commit, the commands, the pass and fail counts and each failure's cause. Rerun once on the final release candidate after 2134 and the replay work land.
