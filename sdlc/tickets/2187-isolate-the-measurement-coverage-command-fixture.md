---
priority: 1
---

# 2187 — Isolate the measurement coverage command fixture

Status: OPEN.

Milestone: 0.9.3

## Outcome

The existing measurement coverage command fixture must exercise its controlled Cargo/coverage replies regardless of the host tool installation. Preserve environment scoping, suite exit capture and incomplete-collection assertions.

- Evidence: Two cases of test_coverage_scopes_compiler_environment_and_marks_failed_collections failed in second rehearsal39078684323 at7e4ede39a6fd54815a9a6dd0db67bcd8a554db86. They created a fake cargo but invoked real Cargo, which failed with no such command llvm-cov. Python reported1259passed,8skipped,3failed total; the other failure belongs to2188.
- Changes: claims `tests/test_measurement_coverage_contract.py` `CHANGELOG.md`. Keep actual measurement harness behavior unchanged unless the fixture establishes a product defect; report any additional claimed file before editing. Existing actual coverage data remains tied to its original revisions, not this fake-command contract.
- Proof: retain the actual Linux rehearsal RED, reproduce the named isolation boundary on current main through a build job, correct it with the smallest existing outside-in contract, and run affected checks. Fresh review, a combined landing gate and actual-main full check precede a new complete nonpublishing rehearsal.
- Keeps: offline isolation, dependency pinning, existing accepted product fixes and release-only triggers. No broad fixture framework or unrelated refactor.

## Reproduction

- Command: `make test-python` in qualification/python-lane, job117531654838 of https://github.com/genomoncology/biomcp/actions/runs/39078684323 .
- The coordinator retains its complete raw log in temporary scratch. Main remains7e4ede39a6fd54815a9a6dd0db67bcd8a554db86; ordinary landings stay held except these reviewed repairs.
