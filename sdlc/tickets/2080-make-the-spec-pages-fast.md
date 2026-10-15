# 2080 — Make the spec pages fast

Status: OPEN.

Milestone: 0.9.3

Depends on: 2071

## Outcome

`make spec` takes 200 s or less on the build host, with no retry back-off sleeps, no block over 40 lines, and no fixture scripts.

## Evidence

- `make spec` takes about 11 minutes and waits more than it computes: simulated provider failures sleep through retry back-off, and the section-outcomes page alone takes about two minutes (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 8).
- 22 blocks exceed 40 lines; the runner is 25 KB of bash with per-page fixture scripts.

- Starts from: 2079's first batch (replay server and golden suite); it starts once that batch lands, not when all of 2079 does.
- Keeps: the pages as reader-facing specifications with `like` and `expect=` assertions.
- Changes: claims `scripts/run-specs.sh` `scripts/prepare-spec-artifacts.py` `spec/fixtures/**` `tests/golden/spec/**` `tests/test_article_spec_fixture_lifecycle.py` `tests/test_ctgov_spec_fixture_lifecycle.py` `tests/test_disease_survival_fixture_lifecycle.py` `tests/test_json_error_fixture_lifecycle.py` `src/sources/mod.rs` `src/entities/article/search/deadline.rs` `Makefile` `CHANGELOG.md`, and each spec page it edits, named on this ticket before the first push.
  - A test setting in `src/sources/mod.rs` (`retry_sleep_duration`) turns retry back-off to zero, and the article deadline in `src/entities/article/search/deadline.rs` takes an injected clock for the fixtures.
  - Move JSON assertions and request-log arithmetic into golden cases under `tests/golden/spec/`, beside 2079's cases.
  - Cut every block over 40 lines.
  - Replace the fixture scripts with the replay server, and delete the four fixture lifecycle tests with them.
  - The `Makefile` `stress` target runs `tests/test_disease_survival_fixture_lifecycle.py` today; point it at the replay server's own test instead, so the stress lane keeps a fixture-server case.
  - One Internal changelog bullet.
- Proof: `make spec` wall time on the build host at 200 s or less, against the 2069 baseline's 657 to 668 s ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)); runner line count before and after; no block over 40 lines; `make stress` passes. A page that cannot reach the target without losing a reader-facing claim moves its JSON checks into golden cases or goes.
- Defers: nothing.
