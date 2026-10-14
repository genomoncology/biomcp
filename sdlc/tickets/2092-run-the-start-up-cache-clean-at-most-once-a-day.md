# 2092 — Run the start-up cache clean at most once a day

Status: OPEN.

Milestone: 0.9.3

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

Executed on the branch `ticket/2092-startup-clean-daily`, yellow, code head `f036a2772f9936410da5c138d9e16c7cba0057f8`, with this evidence appended after it (status stays OPEN until review). The marker is `<cache root>/http/.last-startup-clean` holding the clean time in epoch milliseconds, read only while the maintenance lock is held, and rewritten after each clean; a missing, unreadable, or malformed marker means the clean is due.

- Red probe: on the tests-only head `3eec6d77314966437080a514775f443b28273e11` (main `1f8f1313f` plus the four tests), `cargo nextest run --no-default-features -E 'test(startup_clean)'` failed all four: the mode-off test at "an expired entry survives construction with the cache mode off", so mode-off scans on main today, and the other three at the marker assertions, since main cleans but never writes the marker.
- Green: on the code head the same four pass; `cargo nextest run --no-default-features -E 'test(cache::) + test(startup_clean)'` passes 177; `cargo clippy --no-default-features --all-targets -- -D warnings` is clean.
- Keeps untouched: `biomcp cache clean` and its options (it calls `execute_cache_clean` directly, not the construction path), the age and size limits, and the maintenance lock, which now also guards the marker read and write.
