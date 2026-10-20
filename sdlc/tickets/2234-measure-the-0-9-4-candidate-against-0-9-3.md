---
priority: 2
---

# 2234 — Measure the 0.9.4 candidate against 0.9.3

Status: OPEN.

Milestone: 0.9.4

Depends on: 2134

## Outcome

Changes: `sdlc/records/2234-candidate-measurements.md`, `CHANGELOG.md`, `sdlc/tickets/2284-isolate-and-reduce-test-profile-rebuild-cost.md`.

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

## Progress

- 2026-10-19 started
- 2026-10-19 next: Preparation identified the unchanged existing helper (SHA256 9c427657dcd0c79a1982aae3f692433a88a9b273f131a4ca62b86962aa4bc9c7) and compatible v0.9.3 gates. No builds or samples ran. Await frozen candidate; collect paired rows with direct main-gate and populated-cache disk commands, retaining new raw evidence durably. Historical 2214 archives are unavailable at recorded paths; retention issue sent to PM.
- 2026-10-19 next: Frozen source 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 passed full main and enabled live replay. Final same-host paired collection against v0.9.3 is authorized and preparing with the unchanged helper.

Keep: experiments/809-biomcp-2234-paired — retain every raw sample, failed attempt, continuation, cache body/accounting and method input through review and release consumers.
- 2026-10-19 next: Blocked on build-host disk headroom below v0.9.3 default cache-retention floor plus one full measurement arm. Four valid old timing rows and the known2220 pressure-eviction failure are retained in809; completed BioMCP builds reclaimed. PM routed further safe headroom request. Resume only missing rows after measured headroom, without product or threshold changes.
- 2026-10-19 next: PM confirmed build-host disk space is free and released the hold. Resume only missing first-old-arm rows plus remaining five arms using unchanged helper and default cache floor, retaining valid earlier timings and the original pressure-eviction failure. Report any further eviction immediately.
- 2026-10-19 next: Preserve valid old-release samples and finish active first old arm, then hold before any candidate/runtime samples. Final Linux raised2283 concurrent cache proof failure on51f5e03; await reviewed repair source and explicit sample reuse disposition. No product or cache-threshold changes.

Keep: experiments/491-biomcp-2234-raw — independently verified backup of the first old-release arm raw archive, SHA256 ec3c8b6d15db08472c5129ba6987d99e4878131a4bcad435ae5641597726c17d.
- 2026-10-19 next: PM approved unchanged runtime source51f5e03. Candidate paired collection resumed in1792454239309-63056; preserve successful first old arm and failures. Final records await all paired rows and regression disposition. Local raw backup491 verified and kept.
- 2026-10-19 next: Paired collection continues with two candidate arms green and second old arm running. Gene edit-test and Rust-touch rebuild costs are higher in both observed pairs; selected gene-test execution is unchanged. Existing Cargo output localizes added cost to compilation/linking. Finish third pair and runtime rows, then obtain PM disposition for persistent regressions; no extra probe or source change.
- 2026-10-19 next: Final Linux raised sole repair2285 for parallel article-source cache command1; cause and traceback pending. Keep measured source51f5e03 and completed samples. Continue naturally uncollected old-release rows through final old arm, then hold before remaining candidate/runtime work. Original baseline cancellation failure stays incomplete and matches the recorded2230 preimage class; no retry.
- 2026-10-19 next: All build arms and runtime collection are complete, with failed old rows and source outliers retained. Two millisecond-scale timing watch rows and fixed small-cache SQLite overhead need PM disposition before final record readiness. No extra samples or probes. Runtime processes all exited zero, but recorded partial/provider-timeout limits remain explicit.

## Test-only qualification disposition

PM approved the 2283 cache-test correction under best-effort admission. Product source remains unchanged, so candidate measurements resume on 51f5e03b218c87cedc82bd9e1a67552bd5ab9df8 while final Linux qualification reruns on the repaired tip after landing. Preserve all valid old-release samples and failures.

## Performance disposition

The PM ruled that user-facing runtime rows slower beyond noise require a cause and fix lane for 0.9.4. Developer rebuild cost does not block this release: record the cost in the final measurement record and changelog. Follow-up 2284 in 0.9.5 owns isolating and reducing the test-profile compilation cost. Final paired collection continues before the go request.
