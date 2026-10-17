---
priority: 1
---

# 2187 — Isolate the measurement coverage command fixture

Status: OPEN.

Milestone: 0.9.3

Reviews: revision e5604a6, accept

## Outcome

The existing measurement coverage command fixture must exercise its controlled Cargo/coverage replies regardless of the host tool installation. Preserve environment scoping, suite exit capture and incomplete-collection assertions.

- Evidence: Two cases of test_coverage_scopes_compiler_environment_and_marks_failed_collections failed in second rehearsal39078684323 at7e4ede39a6fd54815a9a6dd0db67bcd8a554db86. They created a fake cargo but invoked real Cargo, which failed with no such command llvm-cov. Python reported1259passed,8skipped,3failed total; the other failure belongs to2188.
- Changes: claims `tests/test_measurement_coverage_contract.py` `CHANGELOG.md`. Keep actual measurement harness behavior unchanged unless the fixture establishes a product defect; report any additional claimed file before editing. Existing actual coverage data remains tied to its original revisions, not this fake-command contract.
- Proof: retain the actual Linux rehearsal RED, reproduce the named isolation boundary on current main through a build job, correct it with the smallest existing outside-in contract, and run affected checks. Fresh review, a combined landing gate and actual-main full check precede a new complete nonpublishing rehearsal.
- Keeps: offline isolation, dependency pinning, existing accepted product fixes and release-only triggers. No broad fixture framework or unrelated refactor.

## Reproduction

- Command: `make test-python` in qualification/python-lane, job117531654838 of https://github.com/genomoncology/biomcp/actions/runs/39078684323 .
- The coordinator retains its complete raw log in temporary scratch. Main remains7e4ede39a6fd54815a9a6dd0db67bcd8a554db86; ordinary landings stay held except these reviewed repairs.

## Progress

- 2026-10-17 started

## Repair and checks

Restated outcome: both existing coverage contract cases select their controlled Cargo and wrapper commands regardless of host tool installation, while retaining environment-restoration and incomplete-collection assertions. The fixture owns its isolated home and puts its fake commands in that home's Cargo command directory, which the real harness deliberately prefixes. Only the existing fixture and changelog changed; the measurement harness did not.

- The first focused attempt1792280038524-95933 stopped before tests because its job checkout lacked pytest. No contract result was claimed. The successful run prepared dependencies offline with `make sync-python-dev` before direct pytest.
- Job1792280073398-97025 at published basee1aafa815 plus controlled patches: the command-precedence RED placed a failing controlled host Cargo ahead of the old fake location; both existing cases failed with that marker. Final GREEN placed the fixture commands under its isolated home, and both cases passed. Ruff lint/format, whitespace and the existing quality ratchet passed.
- Tested final patch SHA2565bb2f9f86b013ce448c2afc63a6897ec65031ea7415567fbd9a7e731ac1b46c9 equals the committed diff e1aafa815..7900c7cef75a756b910ae7c9a14729435803ba4d; coordinator checked the equality and job result.
- Source root cause is test dependence on the host command directory. Existing actual coverage and runtime measurements retain their source/harness identities and remain applicable; no actual measurement was rerun or relabelled. A focused offline Linux check is required before the next full rehearsal.
