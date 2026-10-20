# Map reusable CI checks to local execution

bin/lint now runs the existing branch changelog-coverage and tracked scratch-file checks. The branch checker recognizes a local symbolic ticket branch, supports a dated section matching the stable Cargo package version, and never revives an older bare section beneath the current release. Development or wrong-version dated sections remain invalid. No Makefile or workflow changed.

M5 preimage job 2291/1792505910947-78109 failed nine outside-in Git/lint fixture cases. The first corrected job 1792506102012-97685 passed 113 focused tests but its real-tree check found historical bare 0.8.4 incorrectly selected beneath dated 0.9.4. That integration failure remains failed. Final source 686c8713968356e2a8a18d2bddba6b25df71e8e5 passed job 1792507023960-82920: all 113 tests, Ruff, ShellCheck, actual-tree coverage of 52 shipped-change tickets and whitespace. The historical-heading regression is included. Raw runs remain under experiment 2102-biomcp-2258-final-live-replay/local-ci-parity/2291.

| CI validation or group | Local execution or explicit disposition |
| --- | --- |
| changes / ci-classify-push | CI optimization; local gates always execute their selected checks. |
| build / prepare-test and artifact preparation | make test preparation in the landing gate. Cargo fetch, checkout, setup, caches and artifact transport prepare execution rather than validate behavior. |
| lint / make lint | Landing lint. |
| rust-tests / test-rust | Landing make test. |
| python-lane / test-python | Landing make test, with sync-python-dev preparation. |
| spec-lane / make spec | Landing specifications. |
| stress-lane / make stress | Deliberately release/manual-only repeated CPU/load workload, outside routine gates. |
| release-panic / optimized build and panic selection | Deliberately release-only optimized-profile verification, outside the routine development graph. |
| full-features | Main full-feature-check covers development profile; release-gate covers optimized profile. |
| generated-sources / regenerate --check | Deliberately release/manual-only pinned protobuf generator/toolchain check. This record does not claim it ran on the M5. |
| Windows contracts and released-binary comparison | Deliberately release/manual platform checks; Linux/macOS cannot exercise Windows semantics. This record does not claim the new local lint executed those checks. |
| repository-contracts / non-binary pytest selection | Routine contracts, required repo-audit and release marker tests retain their existing selection boundaries. |
| strict mkdocs | Existing test-contracts-prepared documentation validation. |
| installer equality | Existing canonical installer checksum contract. |
| version synchronization | Existing specification and directory submission contract checks. |
| tracked no-climb check | Newly invoked from local lint; existing implementation reused. |
| branch changelog coverage | Newly invoked from local lint; source selection corrected for current stable candidates. |
| historical green-CI gh query | Obsolete prior-run provenance assumption; direct release candidate qualification replaces it. No offline local GitHub query is introduced. |

Fresh review must check these dispositions against the actual workflow; table entries describe execution locations or exclusions, not new pass claims. The coordinator merged accepted main 6e664cfa after the developer stopped. This gate implementation lands separately.
