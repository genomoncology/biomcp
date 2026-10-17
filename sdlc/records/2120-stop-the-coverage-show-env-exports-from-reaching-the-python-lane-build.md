# Coverage environment isolation evidence

## Behavior

Coverage instrumentation applies only to the Rust integration and unit suites. Before Python and specification execution, the measurement restores each original exported compiler, cache and linker value and unsets keys introduced by the coverage tool. The outside-in profile pattern remains absolute. Existing coverage groups and failed-suite incomplete reporting remain unchanged.

The implementation captures all keys exported by `cargo llvm-cov show-env`, including wrapper-specific keys, plus the target and incremental settings the measurement changes. Restoration preserves the caller's configuration instead of unconditionally discarding it.

## Controlled reproduction

Current-main job `1792244081625-70575` exercised the actual measurement entry point with tiny compiler and test-command doubles. Both the ordinary and forced-failure cases failed because Python and specification commands inherited instrumentation settings. This component reproduction establishes the environment boundary; it is not a full coverage measurement.

Focused implementation job `1792244657168-39437` passed all three contracts, including restoration of caller values, isolation of introduced exports, absolute profile paths and incomplete reporting after a forced Python failure. Its trailing formatting check required correction; final job `1792244700550-1186` passed formatting, Ruff and tracked-text checks. Fresh review job `1792244914059-59847` passed the three contracts and Ruff checks.

## Actual coverage evidence

Actual instrumented job `1792244253514-84513` used the designated build host with Rust 1.95.0, cargo-llvm-cov 0.9.1 and compatible LLVM 22 tools. Integration execution passed 154 tests. Unit execution passed 1840 tests and failed one GenCC bootstrap case, leaving 2342 unrun. Python passed 1155 tests with four failures and eight skips. One new fixture-marker defect was corrected and verified by the focused jobs; the remaining failures concerned the changelog release membership and obsolete package count. Specification execution exited zero, including four golden checks, while the inherited parser incorrectly counted zero pages.

Every coverage group remained incomplete. No complete coverage percentage follows from this run. Separate repairs address canonical temporary paths (2168), changelog membership (2166), package membership (2159) and summary parsing (2169). A real successful combined run must establish complete coverage; component doubles do not substitute for that evidence.

## Historical receipts

Earlier job `1792205434706-2110573` at `43252305f` established that inherited coverage wrapper exports redirected a Cargo-driving release-smoke contract away from the artifact path it checked. Removing those introduced settings made that contract pass. Historical workflow run `38998134752` checked the earlier implementation at `98bf13b60`; it does not establish qualification for the current code or replace the required local checks.
