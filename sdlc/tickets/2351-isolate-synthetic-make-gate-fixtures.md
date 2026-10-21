# 2351 — Isolate synthetic Make gate fixtures

Status: OPEN.

Milestone: 0.9.4

## Outcome

Synthetic Make tests exercise real gate ordering and failure propagation using fixture-owned commands and environment. They cannot inherit host tools or Make controls. Production preflight and its dedicated tests remain intact.

- Changes: `tests/test_routine_cargo_feature_contract.py`, `tests/test_dependency_advisory_gate.py`, `tests/test_feature_coverage.py`, `tests/test_ci_workflow_contract.py`, `tests/test_lint.py`, `tests/make_fixture.py`, `tests/test_gate_preflight.py`, `CHANGELOG.md`.
- Proof: reproduce the six existing gate-order failures on current main with production tools unavailable and hostile inherited Make/tool settings. A shared isolated fixture must preserve exact stage order, stop at the failing stage, and never invoke hostile commands. Run affected modules through pm job. No production changes or real-tool installation to conceal fixture leakage.

## Evidence

- Coordinator restatement: remove repeated host-environment leakage from synthetic recursive Make fixtures, rather than repairing each host or weakening real gates.
- Read-only RCA found the same structural class as 2343: replacing a Make recipe retains its prerequisites. Six cases in `test_landing_commands_run_ordered_checks_and_stop_on_failure` reached real gate-preflight and produced empty expected traces. The fixture also retains direct replay-check invocation and inherits Make controls. Related fixtures use inconsistent environment construction.
- Actual rehearsal 39686850300 Python job 119524809502: six failures, 1983 passed, 12 skipped. Yellow release-evidence/0.9.4/rehearsal-2330/39686850300/python-failed.log records missing nextest/deny/actionlint and incompatible ShellCheck at lines 5899–5902; failed case inventory at 3766–3780. All other actual outcomes remain historical.
- Start under the reversible assumption that this test-only release-blocker repair serves the approved release outcome; PM receives the new work and may reorder or overturn it. Do not expand production behavior.

- PM reply m3485 approves this in 0.9.4 and requires folding 2343's one-off isolation into the shared helper. Developer restatement: preserve real orchestration and failure stopping while owning Make controls, home/config and substitute tool boundaries. Actual claim includes tests/make_fixture.py, test_lint.py and the four inventoried synthetic Make modules; no production changes.
- Current-main RED j4206 executes pristine 3b73f04f5370b86b57f1cb2a41cff8af57376a22 on Gator with disk2/memory2 via `bash /tmp/biomcp-2351-red.sh`: after make sync-python-dev, `BIOMCP_TOOL_DIR=<fixture cargo exits127> .venv/bin/python -m pytest tests/test_routine_cargo_feature_contract.py -k landing_commands -q` fails all six active cases, with two expected context skips and empty stage traces from real preflight. Applying only hostile Make regression additions reproduces six failures and inherited Make configuration execution. Both pytest phases and the job wrapper exited1; this is actual failed RED evidence, not repaired success. The ordinary complete-host baseline j4197 passed and does not disprove host leakage. Retain exact recipe and raw j4206.log with proof archive.
