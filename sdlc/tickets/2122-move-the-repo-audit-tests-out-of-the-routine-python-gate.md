---
priority: 3
---

# 2122 — Run whole-repository audits in local qualification

Status: OPEN.

Milestone: 0.9.4

## Outcome

Retained whole-repository audits run in an explicit required local qualification stage, while routine Python tests avoid redundant whole-tree scans and retain focused behavior checks.

Depends on: 2094
Depends on: 2156

- Starts from: whole-tree checks in `tests/test_quality_ratchet_contract.py`, `tests/test_capture_receipts.py`, `tests/test_zero_coupling_ratchet.py`, `tests/test_cli_surface_contract_ratchet.py`, shared pytest setup and Makefile selections.
- Keeps: meaningful audits and their failing exit codes; focused malformed-input and wrapper-behavior tests stay routine using small fixtures.
- Changes: claims `tests/test_quality_ratchet_contract.py` `tests/test_capture_receipts.py` `tests/test_zero_coupling_ratchet.py` `tests/test_cli_surface_contract_ratchet.py` `tests/conftest.py` `Makefile` `AGENTS.md` `CHANGELOG.md`. Mark actual whole-tree checks rather than moving the old six-test list wholesale. Connect their explicit command to required postmerge qualification. Name runner-contract files before editing them. Keep release-only long audits in candidate qualification and run them when covered behavior changes.
- Proof: through the build job, display routine and audit selections, show each retained audit runs in its declared required stage, and plant an audit violation that makes that stage fail. Compare routine timing on matching host settings.
- Defers: long audit execution to the declared long-suite policy.

PM decision: 2026-10-17. Manual hosted repository-contract jobs cannot replace required local coverage. This work follows the parallel local runner and does not depend on the withdrawn automatic-main cache work.

PM decision: 2026-10-17. Moved to 0.9.4. Ian ruled that 0.9.3 ships once every bug is fixed; this ticket improves tests, tooling or dependencies and fixes no user-facing bug. Rebase onto main before resuming; earlier reviews cover the old base only.
