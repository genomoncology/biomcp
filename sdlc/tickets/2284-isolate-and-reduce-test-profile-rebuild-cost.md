---
priority: 2
---

# 2284 — Isolate and reduce test-profile rebuild cost

Status: OPEN.

Milestone: 0.9.4

Depends on: 2234

## Outcome

Identify which changes between 0.9.3 and 0.9.4 raised test-profile compilation and linking cost, then reduce the gene edit-test and Rust-touch rebuild cost while retaining useful test coverage and development behavior.

Changes: `src/entities/article/variant_search.rs`, `sdlc/records/2284-rebuild-attribution.md`, `CHANGELOG.md`.

## Evidence

The first two measurement pairs in 2234 show a repeated development rebuild slowdown. The first canonical gene edit-test row is 53.9 seconds for 0.9.3 and 63.2 seconds for the candidate. The second pair's outer elapsed values are 39.823 and 58.952 seconds. Its Cargo test-profile compile values are 37.97 and 57.08 seconds, while the same 16 selected tests execute in 0.034 seconds on both. Rust-touch rebuilding is also slower in the second pair. These observations locate the cost in rebuilding; they do not identify a particular source change. Use the final paired table and retained raw evidence in `sdlc/records/2234-candidate-measurements.md` when work starts.

Ian moved this rebuild-cost work into 0.9.4 on 2026-10-20. The separate runtime ruling requires disposition before release. The PM subsequently accepted the measured warm-variant and cached-GWAS timing increases and small-cache SQLite overhead for 0.9.4, with timing cause unknown and no runtime lane; the final 2234 record preserves those exceptions.

## Scope

Start with the existing Cargo output, toolchain, profile, compiler-cache and source-change evidence. Name the bounded attribution experiment and the resulting implementation paths before changing code. Use the existing measurement method. Preserve release profile settings and outside-in behavior unless the measured cause requires an independently reviewed change.

## Proof

Report the identified cause and the changed paths. Compare repeated gene edit-test and Rust-touch rebuild rows before and after on the same build host, toolchain, profile and cache settings. Retain raw samples, separate compilation/linking from selected test execution, and run the affected behavior checks. Do not claim a saving from a changed workload or omitted coverage.

## Approved attribution experiment

Retained cold-build timing identifies the library test binary as the critical path; gene touch rebuilds only this package and executes the same sixteen tests. Profiles are unchanged. Increased concrete async article-search futures are a hypothesis, not an established cause. Test one real BoxFuture boundary at search_variant_articles_identity without changing its arguments, results, body, useful tests or release profiles. Reproduce the current-main before arm before editing source. A negative result rejects the prototype; no wider architecture change follows.

Use the same measurement command, workload order and settings for both arms: cold-dev-build, rust-edit-rebuild, doc-edit-rebuild, edit-test-group. Three alternating pairs on quiet M5 preserve target/cache history. Retain compiler-plus-link totals separately from test execution; a compiler-versus-link subdivision remains unknown. Keep CARGO_INCREMENTAL unset and preserve toolchain/cache/profile settings. Measurements may start only after the coordinator reserves a quiet window. The measurement helper must propagate unexpected failures; use one identical validated helper for both arms and name its revision.
