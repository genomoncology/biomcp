# 2342 — Measure feature coverage by region

Status: OPEN.

Milestone: 0.9.4

## Outcome

tools/collect_feature_coverage.py reports unrun Rust code by source region, not by line. It reads `cargo llvm-cov report --json` and lists every region with a zero count across all test binaries. A closure or branch that shares a line with code that ran then shows up as unrun. The floor and the unreached list use regions, with the baseline reset once and recorded.

- Proof: a fixture where a closure never runs on a line whose other code did run shows that closure in the unreached list; the nightly and release coverage targets pass with the reset region baseline.

## Evidence

- tools/collect_feature_coverage.py:143-146 exports LCOV with `cargo llvm-cov report --lcov` and reports unreached product lines. LCOV line records count a line as covered when any code on it ran.
- On 2026-10-21, Gdux's PM reported through Jarvis that line reports hid two untested closures in Gdux. Gdux fixed it by reading the LLVM coverage JSON regions.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
