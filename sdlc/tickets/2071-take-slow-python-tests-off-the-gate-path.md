# 2071 — Take slow Python tests off the gate path

Status: OPEN.

Milestone: 0.9.3

Depends on: 2075

Reviews: revision 12f3665, accept

Reviews: revision 12f3665, accept

## Outcome

No test in `make test`'s Python lane compiles BioMCP or reads the developer's own HTTP cache.

## Evidence

- `make test` takes 646 s and 640 s on the build host, and the Python lane 476 s and 478 s of it (`sdlc/records/2069-0-9-3-baseline.md`). The study's stand-alone run found the lane's floor in three files: the cold `cargo package` and check test (426 s), a trial search stuck in the HTTP cache's start-up clean because 22 test files set no `BIOMCP_CACHE_DIR` (381 s), and the ratchet self-tests (427 s) (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3).

- Starts from:
  - `tests/test_source_package_boundary.py`, `test_verified_source_package_is_zero_coupled_and_compiles_offline` (line 104): `cargo package` with verification, then a cold `cargo check` of the unpacked crate.
  - `tests/test_short_unix_socket_contract.py`, `test_socket_case_ignores_a_long_ambient_tmpdir` (line 16): runs `cargo test --lib` for a Rust test at `src/cache/clear.rs:738` that already runs in the nextest archive. That test makes its socket with `tempdir_in("/tmp")`, which `test_socket_fixture_uses_unique_short_owned_root` in the same file already pins.
  - `tests/conftest.py`, which has no cache fixture today.
  - `tests/test_biomcp_ci_contract.py:80-90`, which asserts the default cache folder of `tools/biomcp-ci`.
  - `Makefile`, the Python lane target (`test-python`, which 2075 splits out of `test`) and `release-gate`, as they stand after 2075. `tests/test_upstream_planning_analysis_docs.py:1178` forbids `BIOMCP_CACHE_DIR` in the `Makefile`, so the cache folder is set in pytest, not there.
- Keeps: every product behavior. The routine lane keeps the packaged crate's file-count and zero-coupling checks.
- Changes: claims `tests/conftest.py` `tests/test_biomcp_ci_contract.py` `tests/test_source_package_boundary.py` `tests/test_short_unix_socket_contract.py` `tests/test_python_lane_cache_isolation.py` `Makefile` `RUN.md` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`.
  1. `tests/conftest.py`: an autouse fixture sets `BIOMCP_CACHE_DIR` to `tmp_path / "biomcp-cache"` with `monkeypatch.setenv`. Register a `release_gate` marker next to `needs_binary`.
  2. `tests/test_biomcp_ci_contract.py`: clear `BIOMCP_CACHE_DIR` in the tests that assert the wrapper's default folder.
  3. Split the packaging test. The routine test runs `cargo package --no-verify` and keeps the member-count and zero-coupling checks. A new test marked `release_gate` runs the cold `cargo check --locked --offline` of the unpacked crate.
  4. `Makefile`: the routine Python lane adds `-m "not release_gate"`; `release-gate` runs `pytest -m release_gate` after `make test`. Nothing in CI runs `make release-gate` today, so the packaged-crate check runs when a release is cut; that is accepted.
  5. Delete `test_socket_case_ignores_a_long_ambient_tmpdir` and keep the source check.
  6. Add `tests/test_python_lane_cache_isolation.py`: one binary command that builds the HTTP client, with `HOME` and `XDG_CACHE_HOME` pointed at a sentinel folder, then assert the sentinel holds no `biomcp` folder.
  7. `RUN.md` says `make release-gate` runs the packaged-crate check.
  8. One Internal changelog bullet.
- Proof: on the build host, `tools/measure-0-9-3-baseline <clone> --only make-test` before and after (baseline 646.3 / 640.0 s, Python lane 476.0 / 477.5 s); `uv run --no-sync pytest tests/ -n 4 --dist loadfile --durations=30` before and after, with the trial-search detail test under 10 s; no routine test calls `cargo test` or a verifying `cargo package`; `make release-gate` runs and passes the packaged-crate check; the isolation test fails when the autouse fixture is removed.
- Defers: the ratchet self-tests go to 2076, which deletes them; until it lands they set the lane's floor, so this ticket promises no lane time. The start-up cache clean itself goes to 2092. The `make test` target of 200 s or less ([decision](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md)) belongs to 2094, which closes the chain after this ticket, 2076, 2080 and 2081.
- Order: after 2075, which splits the `Makefile` `test` target. Safe alongside 2073.
