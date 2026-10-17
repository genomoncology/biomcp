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
