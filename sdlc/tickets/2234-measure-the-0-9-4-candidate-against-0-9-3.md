---
priority: 2
---

# 2234 — Measure the 0.9.4 candidate against 0.9.3

Status: OPEN.

Milestone: 0.9.4

Depends on: 2134

## Outcome

- Changes: `sdlc/records/2234-candidate-measurements.md` and this ticket during measurement; product edits require a separate declared ticket.

The release measurement compares v0.9.3 and the 0.9.4 candidate on one build machine for test time, spec time, full main check, development build time, first-run and warm command times, and cache size on disk, and names every regression with its cause.

## Evidence

The plan's release exit requires this comparison. 2134 moves the cache into one database, so start-up time, warm reads and disk use all change.

## Keeps

Reuse the 0.9.3 measurement method and its rows. Measure in alternating pairs with load recorded, because the build machine is shared. Add no new harness.

## Proof

One record with the paired table and raw samples. Any row more than ten percent worse gets its own ticket or a stated reason before the release.

## Preparation brief

Restated outcome: compare the final candidate and v0.9.3 using the existing measurement rows and method on the same configured build host, with alternating pairs and load recorded. Cache migration is landed at 87138f6ac470f6204bb3899c14e5ac7fe7e74dcc. Replay slices and the approved article recovery fix remain pending, so no current snapshot is the final measurement candidate.

Begin by reading the existing release measurement records and retained helpers. Identify the exact commands, rows, cache states, compiler/profile settings and raw artifacts to reuse; prepare the v0.9.3 baseline if useful. Report the method and any incompatibility before collecting candidate measurements. Do not invent a new harness, change product code, run whole suites merely to explore, or label preliminary data final. Final paired measurements begin only after the coordinator supplies the source-frozen candidate. Preserve raw evidence and source identities for the final record.

## Progress

- 2026-10-19 started
