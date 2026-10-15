# 2076 — Replace the ratchet scripts with compiler lints

Status: OPEN.

Milestone: 0.9.3

Depends on: 2071
Depends on: 2074
Depends on: 2075

## Outcome

The size, line-cap and wait audits and their three inventories are gone, clippy limits function length and refuses a new sleep in production code, and ruff refuses a new `time.sleep` in tests.

## Evidence

- On current main, 35 of 701 non-merge commits since 2026-10-01 touched only `tools/*.json`: 25 the size inventory, 8 the zero-coupling file, 3 the line-cap allowlist.
- `tools/check-quality-ratchet.py` holds about 22 checks. Most read code, not baseline files (CLI surface, documentation coverage, remote resource bounds), so deleting the whole script would drop real checks. The size audit limits files to 1000 lines (`tools/rust-source-size-inventory.json`, 56 entries); `tools/test-wait-inventory.json` records 74 Rust and 6 Python waits.
- The ratchet self-tests take 427 s of the Python lane (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3).
- PM decision 2026-10-14: a per-function `clippy::too_many_lines` limit replaces the 1000-line file rule, since clippy has no file-length rule. Existing production sleeps are allowed one by one, each with a reason. Every other audit in `tools/check-quality-ratchet.py` stays.
- PM decision 2026-10-14: existing test sleeps are allowed as a group until 2081 removes them, so the lint lands without editing test files the second lane owns. `tools/check-zero-coupling.py` and its baseline stay, so re-stamp commits for that file can still happen; this ticket removes the size and line-cap re-stamps only.

- Starts from: `tools/check-quality-ratchet.py` (`check_rust_source_size` at line 359, `check_cli_line_cap` at line 836) with `tools/rust-source-size-inventory.json` and `tools/cli-line-cap-allowlist.json`; `tools/check-test-wait-ratchet.py` with `tools/test-wait-inventory.json`; the `lint` target and the `repository-contracts` step that runs the wait ratchet.
- Keeps: every other audit in `tools/check-quality-ratchet.py`; `tools/check-zero-coupling.py`; the capture-receipt manifest.
- Changes: claims `tools/check-quality-ratchet.py` `tools/check-quality-ratchet.sh` `tools/check-test-wait-ratchet.py` `tools/rust-source-size-inventory.json` `tools/cli-line-cap-allowlist.json` `tools/test-wait-inventory.json` `tools/update-rust-source-size-inventory` `tests/test_quality_ratchet_contract.py` `tests/test_test_wait_ratchet_check.py` `.github/workflows/ci.yml` `bin/lint` `Makefile` `Cargo.toml` `clippy.toml` `pyproject.toml` `src/lib.rs` `src/cache/migration.rs` `src/cli/system/batch.rs` `src/cli/system/dispatch.rs` `src/entities/article/assets.rs` `src/entities/article/graph/citation_evidence.rs` `src/entities/article/variant_search.rs` `src/entities/disease/search.rs` `src/entities/gene/gencc.rs` `src/entities/variant/clinvar.rs` `src/sources/clingen_cspec.rs` `src/sources/gencc.rs` `src/sources/gencc/store.rs` `src/sources/mod.rs` `src/sources/rate_limit.rs` `CHANGELOG.md`.
  - Delete the size, line-cap and wait audits, their three inventories, the inventory updater and their tests.
  - Add `clippy::too_many_lines` with `too-many-lines-threshold` set to the longest function on main.
  - Add `disallowed-methods` for `std::thread::sleep`, `tokio::time::sleep` and `sleep_until`, allowed with a reason at each existing production call. The 14 production files named above are the ones `tools/test-wait-inventory.json` lists outside test modules.
  - Existing test sleeps stay allowed until 2081 removes them: `src/lib.rs` gets `#![cfg_attr(test, allow(clippy::disallowed_methods))]`, and ruff's `per-file-ignores` lists the Python test files that sleep today. 2081 deletes both allowances when its last batch lands.
  - Ban `time.sleep` in tests through ruff's banned-api.
  - One Internal changelog bullet.
- Proof: on the build host, `make lint` and Python-lane times before and after; clippy fails on a planted 1-line-over function and a planted sleep in production code; ruff fails on a planted `time.sleep` in a new file under `tests/`; `make lint` passes on the branch.
- Defers: a file-length rule (clippy has none).
- Order: after 2075 and 2074 (`.github/workflows/ci.yml`, `bin/lint`). It edits the `Makefile` `lint` target, so it lands after 2070, 2071 and 2075. PM decision 2026-10-14: per [the decision to delete tests boldly](../decisions/2026-10-14-delete-tests-boldly-and-split-the-suites.md), this ticket now lands before 2072; it deletes `tools/rust-source-size-inventory.json`, so 2072 drops its edit to that file, and 2072 rebases onto this ticket's `Cargo.toml`.
