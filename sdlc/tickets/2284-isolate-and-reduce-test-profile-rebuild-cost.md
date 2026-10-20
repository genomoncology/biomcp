---
priority: 2
---

# 2284 — Isolate and reduce test-profile rebuild cost

Status: OPEN.

Milestone: 0.9.4

Depends on: 2234, 2294, 2296

Reviews: revision 7b56fb8, accept

Reviews: revision b8eb64d, accept

## Outcome

Disposition the measured test-profile rebuild slowdown through the approved bounded attribution and single prototype. Land a reduction only if the controlled comparison supports it; otherwise reject the prototype and record that the source cause remains unknown, as the PM explicitly approved.

Changes: `sdlc/records/2284-rebuild-attribution.md`, `CHANGELOG.md`, `sdlc/records/2299-root-cause-of-test-only-check-failures.md`, `sdlc/tickets/2299-keep-a-failed-test-cases-state.md`, `sdlc/tickets/2300-share-the-best-effort-cache-reuse-helper.md`, `sdlc/tickets/2301-finish-the-paused-clock-repairs-and-forbid-the-mix.md`, `sdlc/tickets/2302-replace-fixed-test-deadlines-with-event-waits.md`, `sdlc/tickets/2303-share-one-rust-test-http-server.md`, `sdlc/tickets/2304-add-the-repo-wide-contract-tests-to-the-local-command.md`.

## Evidence

The first two measurement pairs in 2234 show a repeated development rebuild slowdown. The first canonical gene edit-test row is 53.9 seconds for 0.9.3 and 63.2 seconds for the candidate. The second pair's outer elapsed values are 39.823 and 58.952 seconds. Its Cargo test-profile compile values are 37.97 and 57.08 seconds, while the same 16 selected tests execute in 0.034 seconds on both. Rust-touch rebuilding is also slower in the second pair. These observations locate the cost in rebuilding; they do not identify a particular source change. Use the final paired table and retained raw evidence in `sdlc/records/2234-candidate-measurements.md` when work starts.

Ian moved this rebuild-cost work into 0.9.4 on 2026-10-20. The separate runtime ruling requires disposition before release. The PM subsequently accepted the measured warm-variant and cached-GWAS timing increases and small-cache SQLite overhead for 0.9.4, with timing cause unknown and no runtime lane; the final 2234 record preserves those exceptions.

## Scope

Start with the existing Cargo output, toolchain, profile, compiler-cache and source-change evidence. Name the bounded attribution experiment and the resulting implementation paths before changing code. Use the existing measurement method. Preserve release profile settings and outside-in behavior unless the measured cause requires an independently reviewed change.

## Proof

Report the identified cause and the changed paths. Compare repeated gene edit-test and Rust-touch rebuild rows before and after on the same build host, toolchain, profile and cache settings. Retain raw samples, separate compilation/linking from selected test execution, and run the affected behavior checks. Do not claim a saving from a changed workload or omitted coverage.

## Approved attribution experiment

Retained cold-build timing identifies the library test binary as the critical path; gene touch rebuilds only this package and executes the same sixteen tests. Profiles are unchanged. Increased concrete async article-search futures are a hypothesis, not an established cause. Test one real BoxFuture boundary at search_variant_articles_identity without changing its arguments, results, body, useful tests or release profiles. Run the PM-approved exact candidate before arm before editing source. A negative result rejects the prototype; no wider architecture change follows.

Use the same measurement command, workload order and settings for both arms: cold-dev-build, rust-edit-rebuild, doc-edit-rebuild, edit-test-group. Three alternating pairs on quiet Gator preserve target/cache history. Retain compiler-plus-link totals separately from test execution; a compiler-versus-link subdivision remains unknown. Keep CARGO_INCREMENTAL unset and preserve toolchain/cache/profile settings. Measurements may start only after the coordinator reserves a quiet window. The measurement helper must propagate unexpected failures; use one identical validated helper for both arms and name its revision.

PM decision on 2026-10-20: the exact before-arm baseline is reviewed combined candidate3d9472ecc2b4db2d68eee81d54c65453a9804806. The comparison may run in parallel with its serial landing checks. Both arms use the same Gator host, toolchain, cache and validated helper; the only after-arm source change is the boxed-future prototype. Formal clones exclude this ticket’s SDLC metadata. No timing begins before the PM relays Jarvis’s drained Gator hold. If runtime changes before landing, remeasure only affected rows. This supersedes the prior current-main/M5 location condition.

PM clarification: keep3d9472ecc as the relative experiment baseline while2296 fixes fulltext license reporting. That narrow fix does not invalidate these pairs. Remeasure only if it changes Cargo files, build profiles or the code this prototype changes. Run2296 on the M5 or Yellow; Gator stays exclusive to the comparison once drained.

## Final PM stopping rule

The PM accepted rejection of the sole BoxFuture prototype and forbids a second prototype. One additional native Cargo test-profile attribution compares0.9.3 and current main on the same host. If no crate/unit explains substantial growth, close this ticket with the negative result, no fix and no further work. The completed same-host main test unit differs by only0.33seconds, so the earlier slowdown was not reproduced; machine conditions during2234 remain a possible explanation, not an established cause. The added2294 dependency keeps the accepted experiment baseline ancestry out of main until its owning source ticket lands.

## Progress

- 2026-10-20 started

Keep: experiments/2113-biomcp-2284-rebuild-attribution — retain the rejected prototype comparison, raw arms and source identities.

Keep: experiments/2288-biomcp-2284-unit-attribution — retain native unit timings, exact inputs and completed job evidence.

The PM directs this administrative landing to carry the root-cause record and six authorized follow-up tickets from tickets/pm-rca at3819a97abb142b7d57f71eada2e8a4031b9c65a9. Those tickets remain open; this landing claims no implementation of them.
