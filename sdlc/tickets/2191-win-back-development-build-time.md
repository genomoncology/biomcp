---
priority: 2
---

# 2191 — Win back development build time

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 85fb4bc, accept

Reviews: revision 3c2dca6, accept

## Outcome

A fresh development build and a rebuild after a Rust change are at least as fast as v0.9.2's on the same build machine, or the record names what costs the time and why it stays.

- Evidence: the 0.9.3 measurement in `sdlc/records/2069-0-9-3-baseline.md` on one build machine: fresh development build 51.5 s in v0.9.2 and 84.0 s in 0.9.3; Rust-touch rebuild 39.3 s and 55.3 s. Shared host load limits the causal claim, but both rows moved the same way in both 0.9.3 collections.
- Starts from: re-measure both rows with the existing harness in alternating old/current pairs (at least three pairs) on the shared build machine, recording host load per sample; no idle window is required, then find the cause with `cargo build --timings` and the existing LLVM IR line counts (6.90 M to 7.20 M lines; function copies 138,788 to 143,558). Suspects to check, not assume: the 2171 dev profile, crates added since v0.9.2, and generic code that grew more copies.
- Changes: `sdlc/records/2191-development-build-time.md` `sdlc/tickets/2191-win-back-development-build-time.md`. Source changes require a measured cause and an expanded exact claim first.
- Proof: paired before and after numbers for both rows against v0.9.2, with raw samples, and the routine test and spec targets still met.

PM decision: 2026-10-18. The build machine is shared and Jarvis declined holds for other teams, so this ticket uses alternating pairs with recorded load instead of an idle window. Use `cargo build --timings` shares to locate the cost, since relative shares are less sensitive to load than wall time.

## Progress

- 2026-10-18 started
- 2026-10-18 next: The first idle-host attempt ran no measurement because three foreign build-job holders were active. PM superseded the idle requirement: use at least three alternating old/current pairs with per-sample load and Cargo timing shares. Two current BioMCP targeted jobs2216/2199 run first; begin this finite measurement collection when own jobs no longer overlap. Never stop foreign jobs or call these shared-load samples idle.

Coordinator scope: immutable current product `3dc15da27888454aac4a4e43b685f1ebabbbeff5` versus v0.9.2 `4049666f5cce2794f16cdbdf0ded49878e5bb147`, using the existing frozen helper with matching ordinary toolchain/compiler cache/linker settings and incremental unset. Measure fresh-development and Rust-touch rows in three alternating old/current pairs, retaining wall/user/system/RSS, sample load and existing Cargo timing output. Inspect critical units and actual manifest/profile differences before source edits. Existing release-library IR counts are not causal attribution of an all-target development build. Initial claim only `sdlc/records/2191-development-build-time.md` and this ticket; additional source claims require an observed cause. The prepared owned experiment is `2077-biomcp-development-build-cost`; no new benchmark tool or extra broad collection. Record the shared-load limitation and allow the existing outcome to retain a required cost with its explanation.

- 2026-10-18 next: Freeze current product3dc15da27888454aac4a4e43b685f1ebabbbeff5 after the accepted four-ticket batch and its passing actual-main job2156/1792351422343-29859 (408.025s). No sample has run; replace the obsolete prepared9593checkout before the first sample. Complete closed-lane build cleanup first, then run exactly the approved three alternating pairs with per-sample load/Cargo timing shares. All other own M5jobs are held until collection ends. A separately approved GitHub Windows runner does not share this build host and may execute concurrently. Foreign jobs remain allowed and recorded.


## Measured proof

Collection `2191/1792352213116-26987` passed all twelve prescribed commands in 565.098 seconds. Old product `4049666f5cce2794f16cdbdf0ded49878e5bb147` and current product `3dc15da27888454aac4a4e43b685f1ebabbbeff5` used the existing frozen helper and matching ordinary compiler/cache/linker settings, Python 3.12.13 and unset incremental/target overrides. Three pairs ran old/current, current/old, old/current with load recorded around every section. No idle claim is made. All build load ended before analysis and the coordinator released other jobs immediately.

Fresh-build old/current medians are 55.8/54.3 seconds; Rust-touch medians are 39.0/39.4 seconds. The earlier large regression did not reproduce. Cargo timing reports locate the longest retained cost in the library test executable, occupying 72.9–83.7% of fresh-build wall time. The current graph has 465 units against 494 old units. Shared load prevents causal speed claims, and no concrete avoidable bottleneck justifies a source/profile change. Keep required behavior and existing build settings.

The findings record retains every raw timing sample, CPU/system/RSS/load values and Cargo unit comparisons. Disposable raw logs/reports remain owned by experiment 2077 for review. Exact current main already passed full qualification `2156/1792351422343-29859` in 408.025 seconds; reused merged-product landing gate `2216/1792351080065-83519` passed. No additional LLVM, test-suite or performance collection ran. Record formatting was checked with `git diff --check`.
