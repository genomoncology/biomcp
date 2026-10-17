# 2178 — Keep explicit release-smoke mismatch checks independent of live work

Status: OPEN.

Milestone: 0.9.3

## Outcome

The release-smoke explicit-binary mismatch contract reliably rejects the supplied old binary without rebuilding it or waiting on unrelated work.

- Changes: `tests/test_release_smoke_script.py`, `scripts/release-smoke.sh`, `CHANGELOG.md`.
- Keeps: Real release verification, version mismatch diagnostics, explicit binary provenance and the existing no-rebuild assertion remain enforced. Do not hide the failure by blindly increasing the timeout or weakening an assertion.
- Proof: On current main91a862fbb3a8262e3c1a1b649bfd3c2689ae9824, `make main-gate` in job1792265800150-60386 failed `tests/test_release_smoke_script.py::test_explicit_bin_mismatch_is_visible_and_not_rebuilt`: `subprocess.TimeoutExpired` for `bash scripts/release-smoke.sh --bin <fixture>/artifacts/biomcp-old` after30seconds. The Python lane reported1238 passed,9 skipped and one failed. Inspect the real entry path and fixture, identify the concrete wait, and establish a deterministic affected-case RED before production changes. Reuse this actual-main failure; no blind full-gate rerun.
- Routine proof: Run the affected release-smoke module through the build job with its shipped-entry behavior and existing mechanical checks. Reuse unchanged build artifacts; no Rust rebuild is expected for a shell/Python correction.
- Defers: No broader release refactor or new checker. The accepted cache dependency batch continues in parallel and waits only for green main.

## Progress

- 2026-10-17 started
- 2026-10-17 next: Repair the explicit smoke subprocess to follow warning-at30seconds policy with a generous hung-process watchdog; preserve mismatch/no-rebuild checks, identify the concrete fixture wait, then land this main repair first.

Cause and scope before repair: The actual-main failure output ends after both HTTP PASS lines and before plugin PASS, locating the wait at the second HTTP fixture shutdown. The shared fake server sleeps30seconds while fake curl reports healthy without waiting for server readiness; the fixture does not establish readiness before cleanup and its30second sleep meets the shared hard30second watchdog. The exact operating-system signal race is unconfirmed; do not claim it as proved. Repair this shared fixture for both affected cases: install the termination handler before publishing per-port readiness, wait on signals, and require readiness before fake curl claims health. Keep the production release-smoke sequence unchanged. PM ruling: warn at30seconds and retain a generous outer hung-process watchdog. Use the existing SlowTestReporter and scaled timeout helper with a300second watchdog rather than a new reporting system. Preserve every release, mismatch and no-rebuild assertion. A deterministic delayed-start fixture check should fail before the repair and pass with truthful readiness; run only the affected module and its existing mechanical checks.

Ready proof: Job1792266127980-39275 passed both affected release-smoke contracts in2.93seconds on published3a2755 plus the exact test/changelog patch. Ruff, the existing shell wrapper (Bash syntax, ShellCheck and workflow checks), tracked-text checks and the existing quality wrapper passed. No Rust build or behavioral rerun occurred. Source8b5849b3c changes only the shared fixture lifecycle/readiness, the generous scaled watchdog and its changelog entry; the production release-smoke script and original assertions remain unchanged. The actual-main timeout above is the failing proof. Review this complete candidate and land as the sole repair while main is red, then run the full check of actual main before the accepted cache batch.
