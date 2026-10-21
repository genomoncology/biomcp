# 2351 — Isolate synthetic Make gate fixtures

Status: OPEN.

Milestone: 0.9.4

## Outcome

Synthetic Make tests exercise real gate ordering and failure propagation using fixture-owned commands and environment. They cannot inherit host tools or Make controls. Production preflight and its dedicated tests remain intact.

- Changes: `tests/test_routine_cargo_feature_contract.py`, `tests/test_dependency_advisory_gate.py`, `tests/test_feature_coverage.py`, `tests/test_ci_workflow_contract.py`, a shared test helper where needed, `CHANGELOG.md`. Correct the claim after inspecting actual shared boundaries.
- Proof: reproduce the six existing gate-order failures on current main with production tools unavailable and hostile inherited Make/tool settings. A shared isolated fixture must preserve exact stage order, stop at the failing stage, and never invoke hostile commands. Run affected modules through pm job. No production changes or real-tool installation to conceal fixture leakage.

## Evidence

- Coordinator restatement: remove repeated host-environment leakage from synthetic recursive Make fixtures, rather than repairing each host or weakening real gates.
- Read-only RCA found the same structural class as 2343: replacing a Make recipe retains its prerequisites. Six cases in `test_landing_commands_run_ordered_checks_and_stop_on_failure` reached real gate-preflight and produced empty expected traces. The fixture also retains direct replay-check invocation and inherits Make controls. Related fixtures use inconsistent environment construction.
- Actual rehearsal 39686850300 Python job 119524809502: six failures, 1983 passed, 12 skipped. Yellow release-evidence/0.9.4/rehearsal-2330/39686850300/python-failed.log records missing nextest/deny/actionlint and incompatible ShellCheck at lines 5899–5902; failed case inventory at 3766–3780. All other actual outcomes remain historical.
- Start under the reversible assumption that this test-only release-blocker repair serves the approved release outcome; PM receives the new work and may reorder or overturn it. Do not expand production behavior.
