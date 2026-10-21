---
flow: quickfix
priority: 9
---

# Start the CTGov fixture for spec-contracts

`make spec-contracts` includes `spec/surface/trial-retirement.md`, and that page consumes the deterministic CTGov request-log fixture. The runner's `SPEC_CTGOV_FIXTURE_PATHS` registry omits the page, so the conditional fixture startup is skipped and the gate fails with `ctgov request log fixture is not configured` before it can prove the contract.

Add the trial-retirement page to the existing CTGov fixture ownership registry in `scripts/run-specs.sh`. Do not move the page, start public-network work, duplicate the fixture, or change production trial behavior. The runner must start the one existing CTGov fixture before the page, export its existing environment, keep routine/spec-contract fixture cleanup and interruption behavior, and avoid starting it for modes or path sets that do not consume it.

Focused red-green proof belongs in `tests/surface/test_parallel_isolation_contract.py` and `tests/test_ctgov_spec_fixture_lifecycle.py`. The static contract must show that the `spec-contracts` path registry and CTGov ownership registry intersect on trial-retirement and that the fixture starts exactly through the existing single conditional path. The existing copied-workspace lifecycle harness must exercise `spec-contracts` startup and interruption/cleanup, preferably by parameterizing rather than duplicating its current `spec` signal test. Partial copied-workspace runner tests in `tests/test_routine_fixture_recovery.py` and `tests/test_disease_survival_fixture_lifecycle.py` are also authorized to supply the CTGov setup/cleanup stubs now required by the truthful `spec-contracts` ownership registry; they must not weaken runner assertions. The page, fixture scripts, production code, and provider behavior are not authorized to change. All four focused test files, the full routine test inventory, `make spec-contracts`, `make lint`, and `git diff --check` must pass.
