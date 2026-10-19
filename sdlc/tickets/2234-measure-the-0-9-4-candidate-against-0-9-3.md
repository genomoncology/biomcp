---
priority: 2
---

# 2234 — Measure the 0.9.4 candidate against 0.9.3

Status: OPEN.

Milestone: 0.9.4

Depends on: 2134

## Outcome

Changes: `sdlc/records/2234-candidate-measurements.md`, `CHANGELOG.md`.

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

## Final source assignment

The source-frozen candidate is pushed main 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8. Begin the final paired collection only after the coordinator confirms its full main check passes and frees the build-machine slot. Compare with v0.9.3 at 5641de7d7b79b53872f8599dbb4dba22e611dcaf using the unchanged existing helper tools/measure-0-9-3-baseline, SHA256 9c427657dcd0c79a1982aae3f692433a88a9b273f131a4ca62b86962aa4bc9c7. Preserve its established rows and alternating pair method; add no new harness. Label preparation, compiler/linker/profile/cache state, load, raw samples and incomplete failures. Runtime samples use their existing watchdog. The full main comparison runs direct env -u CARGO_INCREMENTAL make main-gate. Cache disk proof includes allocated bytes, file bytes, stats JSON, SQLite, sidecars, blobs and returned data separately. Any worse-than-ten-percent row needs its measured cause, repair or PM disposition before release. No source changes or tag.
