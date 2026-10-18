---
priority: 2
---

# 2191 — Win back development build time

Status: OPEN.

Milestone: 0.9.4

## Outcome

A fresh development build and a rebuild after a Rust change are at least as fast as v0.9.2's on the same build machine, or the record names what costs the time and why it stays.

- Evidence: the 0.9.3 measurement in `sdlc/records/2069-0-9-3-baseline.md` on one build machine: fresh development build 51.5 s in v0.9.2 and 84.0 s in 0.9.3; Rust-touch rebuild 39.3 s and 55.3 s. Shared host load limits the causal claim, but both rows moved the same way in both 0.9.3 collections.
- Starts from: re-measure both rows on an idle build machine with the existing harness, then find the cause with `cargo build --timings` and the existing LLVM IR line counts (6.90 M to 7.20 M lines; function copies 138,788 to 143,558). Suspects to check, not assume: the 2171 dev profile, crates added since v0.9.2, and generic code that grew more copies.
- Changes: claims only what the cause requires; `CHANGELOG.md`.
- Proof: idle-machine before and after numbers for both rows against v0.9.2, with raw samples, and the routine test and spec targets still met.

## Progress

- 2026-10-18 started
