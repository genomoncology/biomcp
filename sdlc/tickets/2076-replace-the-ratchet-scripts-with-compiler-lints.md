# 2076 — Replace the ratchet scripts with compiler lints

Status: OPEN.

Milestone: 0.9.3

## Outcome

Function length and sleeps are enforced by clippy and ruff, the size and wait inventories are gone, and no commit exists only to re-stamp a baseline file.

## Evidence

- On current main, 35 of 701 non-merge commits since 2026-10-01 touched only `tools/*.json`: 25 the size inventory, 8 the zero-coupling file, 3 the line-cap allowlist.
- `tools/check-quality-ratchet.py` holds about 22 checks. Most read code, not baseline files (CLI surface, documentation coverage, remote resource bounds), so deleting the whole script would drop real checks. The size audit limits files to 1000 lines (`tools/rust-source-size-inventory.json`, 56 entries); `tools/test-wait-inventory.json` records 74 Rust and 6 Python waits.
- The ratchet self-tests take 427 s of the Python lane (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 3).
- PM decision 2026-10-14: a per-function `clippy::too_many_lines` limit replaces the 1000-line file rule, since clippy has no file-length rule. Existing production sleeps are allowed one by one, each with a reason. Every other audit in `tools/check-quality-ratchet.py` stays.

- Starts from: `tools/check-quality-ratchet.py` (`check_rust_source_size` at line 359, `check_cli_line_cap` at line 836) with `tools/rust-source-size-inventory.json` and `tools/cli-line-cap-allowlist.json`; `tools/check-test-wait-ratchet.py` with `tools/test-wait-inventory.json`; the `lint` target and the `repository-contracts` step that runs the wait ratchet.
- Keeps: every other audit in `tools/check-quality-ratchet.py`; `tools/check-zero-coupling.py`; the capture-receipt manifest.
- Changes: claims `tools/check-quality-ratchet.py` `tools/check-quality-ratchet.sh` `tools/check-test-wait-ratchet.py` `tools/rust-source-size-inventory.json` `tools/cli-line-cap-allowlist.json` `tools/test-wait-inventory.json` `tools/update-rust-source-size-inventory` `tests/test_quality_ratchet_contract.py` `tests/test_test_wait_ratchet_check.py` `.github/workflows/ci.yml` `bin/lint` `Makefile` `Cargo.toml` `clippy.toml` `pyproject.toml` `src/**` `CHANGELOG.md`.
  - Delete the size, line-cap and wait audits, their three inventories, the inventory updater and their tests.
  - Add `clippy::too_many_lines` with `too-many-lines-threshold` set to the longest function on main.
  - Add `disallowed-methods` for `std::thread::sleep`, `tokio::time::sleep` and `sleep_until`, allowed with a reason at each existing production call.
  - Ban `time.sleep` in tests through ruff's banned-api.
  - The `src/**` claim is wide on purpose until the developer lists the files that get allows; the developer narrows it on the ticket before the first push, so pm stops holding other code tickets behind it.
  - One Internal changelog bullet.
- Proof: on the build host, `make lint` and Python-lane times before and after; clippy fails on a planted 1-line-over function and a planted sleep in a test; ruff fails on a planted `time.sleep` in `tests/`.
- Defers: a file-length rule (clippy has none).
- Order: after 2075 and 2074 (`.github/workflows/ci.yml`, `bin/lint`); after 2072, which edits `tools/rust-source-size-inventory.json`. It edits the `Makefile` `lint` target and `Cargo.toml`, so it lands after 2070, 2071, 2072 and 2075.
