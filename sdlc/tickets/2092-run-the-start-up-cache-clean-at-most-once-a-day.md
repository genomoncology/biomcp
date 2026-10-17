# 2092 — Run the start-up cache clean at most once a day

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision ae9442e, accept

## Outcome

A command run with `BIOMCP_CACHE_MODE=off` never scans the HTTP cache, and with the cache on, the start-up clean runs at most once every 24 hours.

## Evidence

- The iteration-speed study (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, Test times) traced a 381 s trial-search test on the build host to the HTTP cache's start-up clean: the binary read the index of a 19,335-file cache in 58,600 small reads with the maintenance lock held, although the command ran with `BIOMCP_CACHE_MODE=off` and made two requests. Run again with the index in memory, the same command took 1.2 s. Any user with a large cache pays this on every command that builds the HTTP client.
- 2071 isolates the tests from the developer's cache; this ticket removes the cost for users.

- Starts from: `CacheManager::new_at` in `src/cache/manager.rs`, which runs `execute_cache_clean` whenever `try_lock_cache_maintenance` succeeds; `build_http_client` in `src/sources/mod.rs`, which bypasses the cache only for `--no-cache`; `current_cache_mode` in `src/sources/mod.rs`, which already knows `off`; the cache-clean description in `docs/reference/configuration.md`.
- Keeps: `biomcp cache clean` and its options; the age and size limits the clean enforces; the maintenance lock.
- Changes: claims `src/cache/manager.rs` `src/cache/manager/tests/**` `docs/reference/configuration.md` `CHANGELOG.md`.
  - Skip the start-up clean when the cache mode is off.
  - Otherwise run it only when `<cache root>/http/.last-startup-clean` is missing or at least 24 hours old, and touch that file after each clean.
  - One sentence in `docs/reference/configuration.md` saying the automatic clean runs at most daily and `biomcp cache clean` runs it now.
  - One Fixes changelog bullet.
- Proof: routine suite, unit tests in the manager's tests: with the mode off, an expired entry survives construction; with a fresh marker, an expired entry survives; with a marker 25 hours old, the entry is removed and the marker is rewritten; with no marker, the clean runs and writes it. The first two fail on current main.
- Defers: nothing.
- Order: runs alongside anything. It shares only `docs/reference/configuration.md` with 2091, in a different paragraph; land one after the other.

Executed on the branch `ticket/2092-startup-clean-daily`, Linux check host. The work heads `bc5952380be2ead1263b87f82fc4b6bbd612d7e5`..`8ffd240544ed90a6783cf81f02807eb735e4b4d7` were then rebased onto main `a81c61028` (the AGENTS.md gate-name fix; no content conflicts), giving the final code head `2252358182c3da97f1dd9674cfe2a6e7b5e21b73`. Status stays OPEN until review. The marker is `<cache root>/http/.last-startup-clean` holding the clean time in epoch milliseconds, read and rewritten only while the maintenance lock is held; a missing, unreadable, or malformed marker means the clean is due. Writing it creates the HTTP cache directory first through `secure_managed_dir`, because direct construction can reach the clean before anything else has.

- Red probe: on the tests-only head `3eec6d77314966437080a514775f443b28273e11` (main `1f8f1313f` plus the four tests), `cargo nextest run --no-default-features -E 'test(startup_clean)'` failed all four with main's behavior; the mode-off test failed at "an expired entry survives construction with the cache mode off", so mode-off scans on main today.
- First fixed heads failed two lint-only gates: `cargo fmt --check` (fixed in `5061fe5d6f321a73360b0909ed466cc4152728c5`) and the quality ratchet's exact Rust line-count pin (`tools/rust-source-size-inventory.json` now authorizes the manager's growth under ticket 2092 through `tools/update-rust-source-size-inventory`; `src/sources/mod.rs` returned to byte-identical with main by reading the existing `current_cache_mode` instead of adding a helper there).
- Early full-suite green runs on the shared checkout target were stale-compile artifacts: rerun on a private clean target, `stale_marker_tests::put_clears_the_marker_so_revalidation_cannot_carry_it` failed construction because the marker write needed the HTTP directory; `bc5952380` fixes that and all verification below is on the private target.
- Green at the final code head `225235818`: full CI run 38648338151 passed every job; `make lint` passes on the build host. Earlier in the lane, at `bc5952380` before the rebase: the four start-up tests pass; `test(cache::manager) + test(cache_mode) + test(startup_clean)` passes 37; `test(cache::) + test(cache_mode) + test(current_cache_mode)` passes 191; `cargo clippy --no-default-features --all-targets -- -D warnings`, `cargo fmt --check`, and `tools/check-quality-ratchet.sh` are clean.
- The packaged-file contract moved with the new test file: `MAX_PACKAGE_FILES` is 1_418 in `tests/test_source_package_boundary.py` (CI's packaging test observed the 1_418-member archive).
- Keeps untouched: `biomcp cache clean` and its options (it calls `execute_cache_clean` directly, not the construction path), the age and size limits, and the maintenance lock, which now also guards the marker read and write.
