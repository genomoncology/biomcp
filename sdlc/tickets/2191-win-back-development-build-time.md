---
priority: 2
---

# 2191 — Win back development build time

Status: OPEN.

Milestone: 0.9.4

## Outcome

A fresh development build and a rebuild after a Rust change are at least as fast as v0.9.2's on the same build machine, or the record names what costs the time and why it stays.

- Evidence: the 0.9.3 measurement in `sdlc/records/2069-0-9-3-baseline.md` on one build machine: fresh development build 51.5 s in v0.9.2 and 84.0 s in 0.9.3; Rust-touch rebuild 39.3 s and 55.3 s. Shared host load limits the causal claim, but both rows moved the same way in both 0.9.3 collections.
- Starts from: re-measure both rows with the existing harness in alternating old/current pairs (at least three pairs) on the shared build machine, recording host load per sample; no idle window is required, then find the cause with `cargo build --timings` and the existing LLVM IR line counts (6.90 M to 7.20 M lines; function copies 138,788 to 143,558). Suspects to check, not assume: the 2171 dev profile, crates added since v0.9.2, and generic code that grew more copies.
- Changes: claims only what the cause requires; `CHANGELOG.md`.
- Proof: paired before and after numbers for both rows against v0.9.2, with raw samples, and the routine test and spec targets still met.

PM decision: 2026-10-18. The build machine is shared and Jarvis declined holds for other teams, so this ticket uses alternating pairs with recorded load instead of an idle window. Use `cargo build --timings` shares to locate the cost, since relative shares are less sensitive to load than wall time.

## Progress

- 2026-10-18 started
- 2026-10-18 next: The first idle-host attempt ran no measurement because three foreign build-job holders were active. PM superseded the idle requirement: use at least three alternating old/current pairs with per-sample load and Cargo timing shares. Two current BioMCP targeted jobs2216/2199 run first; begin this finite measurement collection when own jobs no longer overlap. Never stop foreign jobs or call these shared-load samples idle.

Coordinator scope: immutable current product `9593a032b534b6252b1133c657c1088f7c5e908f` versus v0.9.2 `4049666f5cce2794f16cdbdf0ded49878e5bb147`, using the existing frozen helper with matching ordinary toolchain/compiler cache/linker settings and incremental unset. Measure fresh-development and Rust-touch rows in three alternating old/current pairs, retaining wall/user/system/RSS, sample load and existing Cargo timing output. Inspect critical units and actual manifest/profile differences before source edits. Existing release-library IR counts are not causal attribution of an all-target development build. Initial claim only `sdlc/records/2191-development-build-time.md` and this ticket; additional source claims require an observed cause. The prepared owned experiment is `2077-biomcp-development-build-cost`; no new benchmark tool or extra broad collection. Record the shared-load limitation and allow the existing outcome to retain a required cost with its explanation.
