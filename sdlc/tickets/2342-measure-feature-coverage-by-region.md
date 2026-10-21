# 2342 — Measure feature coverage by region

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 90acdd6a85addf620e9a4184455d13a76a209974, accept

Landed: a815162

## Outcome

tools/collect_feature_coverage.py reports unrun Rust code by source region, not by line. It reads `cargo llvm-cov report --json` and lists every region with a zero count across all test binaries. A closure or branch that shares a line with code that ran then shows up as unrun. The floor and the unreached list use regions, with the baseline reset once and recorded.

- Changes: `tools/feature_coverage.py`, `tools/collect_feature_coverage.py`, `tools/check-feature-coverage.py`, `tests/test_feature_coverage.py`, `pyproject.toml`, `CHANGELOG.md`, `Makefile`, `docs/reference/release-process.md`, `tools/bootstrap-lint-tools`, `tools/gate-tools`, `bin/lint`, `.github/actions/lane-setup/action.yml`, `tests/test_advisory_bootstrap.py`, `tests/test_lint.py`, `tests/test_offline_gate_contract.py`, `AGENTS.md`.

- Proof: a fixture where a closure never runs on a line whose other code did run shows that closure in the unreached list; the nightly and release coverage targets pass with the reset region baseline.

## Evidence

- Fresh reviewer conditionally accepted product/tooling revision acdc98802a75da5e29b41c2b28d78c1908820f8a after independently reading the full diff and the compiler probe, baseline, floor enforcement, installer and focused-test evidence. Both review findings were fixed: isolate the platform dependency and avoid an unset-HOME shell failure. Acceptance requires the sole declared candidate landing gate to pass before publication. The following coordinator changes affect only this ticket record.

- The integrated Linux collection j2948 passed product build, all selected Rust and Python cases, specifications and JSON export on published main e5c05f8b1eba906625305aa3b8b067185292e2cb with transported tooling patch SHA256 a9d4cba371ce72c38634942da368f7f36086da85ecac19eb50044e502f9a0b81. It reached 97759 of 169642 mapped product regions. The floor was reset once to the measured 57.62664906096368 percent. Gator j3041 passed enforcement of that exact final floor against the same retained JSON, without recollection. Final region parsing and Linux instrumentation flags match the measured patch; subsequent changes add the platform guard, shared instrumentation helper, bootstrap, tests, documentation and measured floor. Rust, Cargo and toolchain inputs remain unchanged.
- PM m3303 makes the floor Linux-only. PM m3309 authorized one additional M5 instrumentation attempt after applying continuous-counter relocation on every platform. Actual j3018 failed with exit 100 on the original quiet-stderr integration case: LLVM still reported a counters section not aligned to its 16384-byte page. Export exited zero but contained zero positive code regions despite four nonempty profile files. This is incomplete profiling, not valid coverage. Darwin collection remains refused and no further attempt was made. Earlier incomplete j2933 and platform-test harness failures j2971/j2977 remain failures; the isolated module-local platform proxy passed j2978.
- Pinned cargo-deny 0.19.4 now comes from the existing policy declaration through verified official archives. M5 j3033 and Gator j3041 proved an isolated actual installation, cached reuse with network access blocked and checkout-cache precedence over Cargo home. M5 j3033 passed 137 affected tests with eight platform skips, Ruff, Bash syntax, ShellCheck and workflow checks. The platform-test proxy leaves the real standard-library platform untouched. Gator j3032 failed before installation because its initial transport exceeded ARG_MAX; corrected file transport supplied j3041. No failed attempt supplies measurement evidence.
- Raw proof is retained on Yellow under data/biomcp/ticket-evidence/2342: linux-baseline-j2948.tar.gz SHA256 098f7d10af1a6fa79c0246f03b20c1c5a7f779d1ee05843c3de110e6b1f10c0e; installer-floor-focused-proof.tar.gz SHA256 6df09d8f6789a03899663222dc0ca0ce69fa520082c882b22e0c2a9e118dd8d4; the bounded Darwin negative-proof archive SHA256 71c9367e26f815e75c308e15b95fbcb94fe601457d5b67e5938d9f8281f6ce6f. The earlier semantic probe archive region-semantics.tar.gz has SHA256 5003ba2e0ffa48620f6b1a86c71aa9420e03e64e6e5879661ff12317f78dedc9.
- Coordinator independently read terminal job statuses and actual logs for j2948, j3018, j3033 and j3041. PM m3336 approves the supported landing recovery: isolate the reviewed no-ff merge, run one direct make landing-gate on its exact candidate, publish main after it passes, record the merge with pm ticket land --merge and clean up. The routine gate and final review remain required; developer proof does not replace them.

- Coordinator restatement before implementation: report mapped executable source regions whose count is zero across test binaries, including an unrun closure sharing an executed line. Preserve product/test exclusions and coverage accounting. Reset the region floor once using the integrated ready-batch source; both nightly and release coverage paths use the shared collector.
- Current-main Gator j2890 at 9998aea0596d4aabdf0776c806c28709135aff74 ran the transported outside-in test with `.venv/bin/python -m pytest tests/test_feature_coverage.py -k unrun_closure -q`. The test failed because the existing report rejects `--json`; that is an interface RED, not independent proof of a hidden closure. The wrapper exits zero after verifying the expected failed test and retains its actual output.
- Gator j2899 provides the separate semantic counterexample on that same published main: pinned rustc 1.95.0, LLVM 22.1.2 and export format 3.1.0 compile a retained single-line closure probe. The unexecuted closure spans line 1 columns 29–49 with count zero, while LCOV reports `DA:1,1` and the called closure has a positive count. The real source, JSON, LCOV and full command/job output are retained in the job's `.cache/region-semantics` for archival on Yellow. Code wholly eliminated by the compiler has no mapped region and is outside the instrumented denominator; the initial unused-closure probe showed this limit rather than the required hidden-region case.
- Fresh design review independently checked the real compiler output and LLVM exporter field/kind definitions. Region source-span union across instances, each function's filename table, and code-region filtering preserve the demonstrated miss without multiplying generic-function instances. The final implementation and actual reset still require their own proof.

- tools/collect_feature_coverage.py:143-146 exports LCOV with `cargo llvm-cov report --lcov` and reports unreached product lines. LCOV line records count a line as covered when any code on it ran.
- On 2026-10-21, Gdux's PM reported through Jarvis that line reports hid two untested closures in Gdux. Gdux fixed it by reading the LLVM coverage JSON regions.
- Ian asked on 2026-10-21 to clear every known issue in 0.9.4.
