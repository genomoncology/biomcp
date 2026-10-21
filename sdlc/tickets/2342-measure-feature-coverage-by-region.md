# 2342 — Measure feature coverage by region

Status: OPEN.

Milestone: 0.9.4

## Outcome

tools/collect_feature_coverage.py reports unrun Rust code by source region, not by line. It reads `cargo llvm-cov report --json` and lists every region with a zero count across all test binaries. A closure or branch that shares a line with code that ran then shows up as unrun. The floor and the unreached list use regions, with the baseline reset once and recorded.

- Changes: `tools/feature_coverage.py`, `tools/collect_feature_coverage.py`, `tools/check-feature-coverage.py`, `tests/test_feature_coverage.py`, `pyproject.toml`, `CHANGELOG.md`.

- Proof: a fixture where a closure never runs on a line whose other code did run shows that closure in the unreached list; the nightly and release coverage targets pass with the reset region baseline.

## Evidence

- Coordinator restatement before implementation: report mapped executable source regions whose count is zero across test binaries, including an unrun closure sharing an executed line. Preserve product/test exclusions and coverage accounting. Reset the region floor once using the integrated ready-batch source; both nightly and release coverage paths use the shared collector.
- Current-main Gator j2890 at 9998aea0596d4aabdf0776c806c28709135aff74 ran the transported outside-in test with `.venv/bin/python -m pytest tests/test_feature_coverage.py -k unrun_closure -q`. The test failed because the existing report rejects `--json`; that is an interface RED, not independent proof of a hidden closure. The wrapper exits zero after verifying the expected failed test and retains its actual output.
- Gator j2899 provides the separate semantic counterexample on that same published main: pinned rustc 1.95.0, LLVM 22.1.2 and export format 3.1.0 compile a retained single-line closure probe. The unexecuted closure spans line 1 columns 29–49 with count zero, while LCOV reports `DA:1,1` and the called closure has a positive count. The real source, JSON, LCOV and full command/job output are retained in the job's `.cache/region-semantics` for archival on Yellow. Code wholly eliminated by the compiler has no mapped region and is outside the instrumented denominator; the initial unused-closure probe showed this limit rather than the required hidden-region case.
- Fresh design review independently checked the real compiler output and LLVM exporter field/kind definitions. Region source-span union across instances, each function's filename table, and code-region filtering preserve the demonstrated miss without multiplying generic-function instances. The final implementation and actual reset still require their own proof.

- tools/collect_feature_coverage.py:143-146 exports LCOV with `cargo llvm-cov report --lcov` and reports unreached product lines. LCOV line records count a line as covered when any code on it ran.
- On 2026-10-21, Gdux's PM reported through Jarvis that line reports hid two untested closures in Gdux. Gdux fixed it by reading the LLVM coverage JSON regions.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
