---
priority: 2
---

# 2284 — Isolate and reduce test-profile rebuild cost

Status: OPEN.

Milestone: 0.9.5

Depends on: 2234

## Outcome

Identify which changes between 0.9.3 and 0.9.4 raised test-profile compilation and linking cost, then reduce the gene edit-test and Rust-touch rebuild cost while retaining useful test coverage and development behavior.

## Evidence

The first two measurement pairs in 2234 show a repeated development rebuild slowdown. The first canonical gene edit-test row is 53.9 seconds for 0.9.3 and 63.2 seconds for the candidate. The second pair's outer elapsed values are 39.823 and 58.952 seconds. Its Cargo test-profile compile values are 37.97 and 57.08 seconds, while the same 16 selected tests execute in 0.034 seconds on both. Rust-touch rebuilding is also slower in the second pair. These observations locate the cost in rebuilding; they do not identify a particular source change. Use the final paired table and retained raw evidence in `sdlc/records/2234-candidate-measurements.md` when work starts.

The PM ruled that this developer rebuild cost is a known 0.9.4 release cost and belongs in the next release. User-facing runtime regressions remain release blockers under that separate ruling.

## Scope

Start with the existing Cargo output, toolchain, profile, compiler-cache and source-change evidence. Name the bounded attribution experiment and the resulting implementation paths before changing code. Use the existing measurement method. Preserve release profile settings and outside-in behavior unless the measured cause requires an independently reviewed change.

## Proof

Report the identified cause and the changed paths. Compare repeated gene edit-test and Rust-touch rebuild rows before and after on the same build host, toolchain, profile and cache settings. Retain raw samples, separate compilation/linking from selected test execution, and run the affected behavior checks. Do not claim a saving from a changed workload or omitted coverage.
