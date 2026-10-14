# 2095 — Limit each routine test to a few seconds

Status: OPEN.

Milestone: 0.9.3

Depends on: 2071
Depends on: 2081
Depends on: 2076
Depends on: 2094

## Outcome

A routine Rust or Python test that runs longer than 10 seconds fails the run, and the full routine suite passes under that limit.

## Evidence

- Ian ruled on 2026-10-14 that every routine test takes a few seconds at most, enforced by the runners once the slow tests have injected clocks ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)). This ticket sets that limit at 10 s.
- Today nextest marks a test slow at 120 s and kills it at 240 s (`.config/nextest.toml`), and pytest sets no per-test timeout though `pytest-timeout` is installed (`pyproject.toml`). The 2069 baseline counts 62 tests over one second. 2071 removes the Python lane's slow cases and 2081 gives the slow tests that stay an injected clock.

- Starts from: `.config/nextest.toml`, which has only `[profile.default]`, so every nextest call uses it: the routine lane, the `Makefile` `stress` target (timeout scale 6), the `Makefile` `verify` target (`nextest run --release ... --run-ignored only`) and the `release-panic` job in `.github/workflows/ci.yml`; `[tool.pytest.ini_options]` in `pyproject.toml`; the stress target's pytest run of `tests/test_disease_survival_fixture_lifecycle.py` (or what 2080 leaves there); the `release_gate` packaged-crate test 2071 adds to `tests/test_source_package_boundary.py` (about 426 s).
- Keeps: the stress lane, `make verify` and the release-panic job, each with today's limit of 120 s marked slow and terminated after the second period.
- Changes: claims `.config/nextest.toml` `pyproject.toml` `Makefile` `.github/workflows/ci.yml` `tests/test_source_package_boundary.py` `CHANGELOG.md`.
  - `[profile.default]` marks a test slow at 5 s and terminates it after the second period, so a routine test over 10 s fails.
  - New `[profile.stress]` and `[profile.release-gate]` keep `slow-timeout = { period = "120s", terminate-after = 2 }`. The `stress` target passes `--profile stress`; `verify` and the release-panic job pass `--profile release-gate`.
  - `pyproject.toml` sets `timeout = 10` for pytest. The `release_gate` test carries `@pytest.mark.timeout(900)`, and the stress target's pytest run passes `--timeout 120`.
  - Fix or delete any routine test the limit catches, under the four-kinds rule.
  - One Internal changelog bullet.
- Proof: a planted Rust test that sleeps 11 s fails `cargo nextest run --no-default-features` and a planted Python test that sleeps 11 s fails the Python lane, while both pass on main; `make test` passes with the limit on; `make stress` and `make verify` pass with their profiles, and `make verify` runs the release-panic job's exact test command.
- Defers: nothing.
