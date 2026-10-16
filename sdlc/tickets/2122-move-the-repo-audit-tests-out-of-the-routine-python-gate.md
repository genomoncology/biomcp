# 2122 — Move the repo-audit tests out of the routine Python gate

Status: OPEN.

Milestone: 0.9.3

## Outcome

The whole-repository audit tests run once per revision in the repository-contracts job, not inside the routine Python lane, so the Python gate step gets shorter without dropping any audit.

## Evidence

- 2116 audit, 2026-10-16: the six costliest Python tests are whole-repository audits: `test_wrapper_writes_summary_artifacts_for_pass_fixture` 88.4 s, `test_dead_code_allowance_audit_passes_for_repo` 79.6 s, `test_repository_historical_inventory_is_exact` 30.8 s, `test_spec_lint_rejects_preparation_owned_cargo_commands` 28.2 s, `test_repository_audit_classifies_every_source_file` 26.4 s, `test_quality_ratchet_runs_whole_surface_cli_contract` 25.9 s (contended upper bounds, about 280 s summed in a 275 s-wall four-worker lane).
- The python-lane gate step averages 4.3 min on green main runs. Expected after the move: about 2 min, about 2.3 runner-minutes saved on each of 535 full green runs, about 20 runner-hours per two weeks.
- The repository-contracts job already runs the Python tests that need no binary on every push.

- Starts from: the six tests above, `Makefile` (`test-python`), `.github/workflows/ci.yml` (python-lane and repository-contracts), `pyproject.toml` markers.
- Decisions:
  - Mark the six tests, and any other whole-repository audit the lane finds over 10 s, with one marker; the routine Python lane skips it and the repository-contracts job runs it.
  - A test that needs the built binary stays in a binary lane; the marker applies only to audits that read the tree.
  - `make test` locally still runs them, so a local closing gate loses nothing.
- Changes: claims `tests/**` (markers only) `pyproject.toml` `Makefile` `.github/workflows/ci.yml` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`.
- Proof: a green CI run where the python-lane gate step time and the repository-contracts time are recorded beside 4.3 min and 1.3 to 1.7 min; every marked test still runs and passes in repository-contracts on that run.
- Order: shares `ci.yml` and `Makefile` with 2094 and 2121; rebase as they land.
