---
priority: 2
---

# 2234 — Measure the 0.9.4 candidate against 0.9.3

Status: OPEN.

Milestone: 0.9.4

Depends on: 2134

## Outcome

The release measurement compares v0.9.3 and the 0.9.4 candidate on one build machine for test time, spec time, full main check, development build time, first-run and warm command times, and cache size on disk, and names every regression with its cause.

## Evidence

The plan's release exit requires this comparison. 2134 moves the cache into one database, so start-up time, warm reads and disk use all change.

## Keeps

Reuse the 0.9.3 measurement method and its rows. Measure in alternating pairs with load recorded, because the build machine is shared. Add no new harness.

## Proof

One record with the paired table and raw samples. Any row more than ten percent worse gets its own ticket or a stated reason before the release.
