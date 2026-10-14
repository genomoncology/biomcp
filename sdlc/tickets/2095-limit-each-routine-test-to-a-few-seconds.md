# 2095 — Limit each routine test to a few seconds

Status: OPEN.

Milestone: 0.9.3

Depends on: 2071
Depends on: 2081

## Outcome

A routine Rust or Python test that runs longer than 10 seconds fails the run, and the full routine suite passes under that limit.

## Evidence

- Ian ruled on 2026-10-14 that every routine test takes a few seconds at most, enforced by the runners once the slow tests have injected clocks ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)).
- Today nextest marks a test slow at 120 s and kills it at 240 s (`.config/nextest.toml`), and pytest sets no per-test timeout though `pytest-timeout` is installed (`pyproject.toml`). The 2069 baseline counts 62 tests over one second. 2071 removes the Python lane's slow cases and 2081 gives the slow tests that stay an injected clock.

- Starts from: `.config/nextest.toml` `[profile.default]`; `[tool.pytest.ini_options]` in `pyproject.toml`; the stress lane's timeout scale in the `Makefile`.
- Keeps: the stress lane and the release-only checks, which keep their own longer limits.
- Changes: claims `.config/nextest.toml` `pyproject.toml` `Makefile` `CHANGELOG.md`.
  - nextest's default profile marks a test slow at 5 s and terminates it after the second period; the stress and release profiles keep today's longer limit.
  - pytest gets `timeout = 10` for the routine lane; release-gate tests keep a longer one through their marker.
  - Fix or delete any routine test the limit catches, under the four-kinds rule.
  - One Internal changelog bullet.
- Proof: a planted test that sleeps 11 seconds fails `cargo nextest run --no-default-features` and a planted Python test that sleeps 11 seconds fails the Python lane, while both pass on main; `make test` passes with the limit on.
- Defers: nothing.
