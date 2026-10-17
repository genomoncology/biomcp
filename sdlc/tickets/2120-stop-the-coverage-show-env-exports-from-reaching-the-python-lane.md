---
priority: 2
---

# 2120 — Scope coverage instrumentation to its Rust suite

Status: OPEN.

Milestone: 0.9.3

Depends on: 2117

## Outcome

The coverage measurement applies show-env exports only to the instrumented Rust suite, clears them before Python and specification execution, and reports complete outside-in and combined coverage only when every required suite succeeds.

- Starts from: `tools/measure-0-9-3-baseline` and the existing implementation branch; 2069 and 2117 recorded the inherited coverage environment failing twenty-six Cargo-driving Python contracts.
- Keeps: 2117's forced-failure behavior; failed or missing suites produce incomplete results rather than percentages. No suite is skipped or weakened to obtain a number; absolute caller-supplied profile directories remain supported.
- Changes: claims `tools/measure-0-9-3-baseline` `CHANGELOG.md`.
- Proof: record the current-main failure using `tools/measure-0-9-3-baseline . --only coverage` through the build job, then run the same entry point on the implementation. Record actual successful exit results for every Rust, Python and specification suite before reporting complete percentages. Repeat the existing forced-failure case and confirm it still reports incomplete with failure propagation.
- Defers: product-line denominator and coverage group reporting to 2123; final comparable measurements to 2082.

PM decision: 2026-10-17. Keep the existing implementation that scopes instrumentation to its consumer; do not alter tests merely to tolerate leaked exports.
