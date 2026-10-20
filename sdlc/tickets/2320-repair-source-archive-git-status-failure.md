---
priority: 1
---

# 2320 — Repair the source archive Git status failure

Status: OPEN.

Milestone: 0.9.4

## Outcome

The source archive check passes while unrelated tests create and remove their temporary directories. Retain the real package-content and source-boundary checks. Do not hide a packaging failure with retries or a skipped assertion.

- Proof: Reproduce the disappearing-directory failure with a controlled concurrent temporary-directory change, make the corrected archive check pass, and run the routine suite before review. The repaired pushed main must pass the full main check before ordinary landings resume.

## Evidence

Actual main 5eaba3028d0b494d441eafd6d6e7e36d34bcc3f3 failed job 2310/1792535942921-65624 with exit 2 in 231447 ms. tests/test_source_package_boundary.py::test_source_package_archive_is_zero_coupled ran cargo package --offline, whose Git status scan failed because temporary root directory .tmp3Av52i disappeared during its directory read (ENOENT). The result retained 1735 passing cases, 17 skips and this one failure. The original result remains failed under wheel-scratch/2310/final-retention/failed-main-1792535942921-65624. Root cause is not yet established; one repair lane owns the bounded investigation.

Changes: `tests/json_error_contract.rs`, `tests/rmcp_client_contract.rs`, `CHANGELOG.md`.
