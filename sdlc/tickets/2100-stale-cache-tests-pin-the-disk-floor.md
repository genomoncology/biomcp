# 2100 — Pin the disk floor in the stale-cache test fixture

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision 3b9ee6d, accept

## Outcome

The three stale-cache tests in `src/cli/stale_json_note_tests.rs` pass on a host whose temporary filesystem has less than 10% free space, because their fixture sets its own disk floor instead of inheriting the production default.

## Evidence

- GitHub issue #291 (2026-10-15) reports that `a_stale_article_search_json_states_the_cache_age_in_meta_notes`, `a_stale_search_all_json_states_the_cache_age_in_meta_notes` and `a_stale_clingen_prefetch_note_reaches_the_gene_card` fail when the filesystem holding their cache has less than 10% free. Disk-pressure eviction removes the cached response, so after the fixture server stops the second request reaches the closed localhost endpoint instead of serving stale data. The reporter A/B tested article search on the v0.9.2 binary: `BIOMCP_CACHE_MIN_DISK_FREE=10%` fails the stale request, and `1B` passes with the cache-age note.
- `StaleNoteEnv` (`src/cli/stale_json_note_tests.rs:166`) sets the source bases and `BIOMCP_CACHE_DIR` in all three constructors (`two_bases`, `bases`, `new`) and restores them on drop, but never sets `BIOMCP_CACHE_MIN_DISK_FREE`. The default is `DiskFreeThreshold::Percent(10)` (`src/cache/config.rs:12`). Checked on main 2026-10-15.
- Ticket 1188 fixed the same gap in the FDA orphan cache test by pinning `1B`, the smallest legal value.

- Starts from: `StaleNoteEnv` and its `Drop` in `src/cli/stale_json_note_tests.rs`.
- Keeps: the production default, the dedicated disk-pressure tests in `src/cache/`, and every other fixture.
- Changes: claims `src/cli/stale_json_note_tests.rs` `CHANGELOG.md`.
  - Each `StaleNoteEnv` constructor records the previous `BIOMCP_CACHE_MIN_DISK_FREE` and sets it to `1B`, the same way it handles `BIOMCP_CACHE_DIR`, so the existing `Drop` restores it.
  - One Fixed changelog bullet that credits the reporter and names issue #291.
- Proof: run the three tests with `BIOMCP_CACHE_MIN_DISK_FREE=100%` exported, which forces eviction the way 1188 reproduced it. All three fail on current main and pass after the change. No production code changes.
- Defers: a shared test-wide disk floor for every fixture that sets `BIOMCP_CACHE_DIR`. File it if a third report arrives.
- Order: any time. It shares only `CHANGELOG.md` with other tickets.

## Amendment 2026-10-15: the whole suite, not three tests

A sweep on main at 679412883 ran the full Rust suite (4,233 tests) three ways. With no settings it passed. With a `cache.toml` setting `max_size = 1` placed in the user config folder it also passed. With `BIOMCP_CACHE_MIN_DISK_FREE=100%` exported, which acts like a full disk, five tests failed: the three from the issue, plus `entities::disease::get::tests::a_stale_disease_card_tells_the_clinician_the_cache_age` and `a_stale_mcp_call_tells_the_clinician_the_cache_age_on_both_channels`. The two disease tests use the shared `test_support::FixtureEnv::scratch_cache_only`, which isolates the cache folder but not the disk floor.

- Changes: also claims `src/test_support.rs`. `FixtureEnv::scratch_cache` records and pins `BIOMCP_CACHE_MIN_DISK_FREE=1B` beside `BIOMCP_CACHE_DIR`, so every test built on `FixtureEnv` stops inheriting the host's free space. `StaleNoteEnv` gets the same pin as above. A test that sets its own floor still wins, because it sets the variable after the fixture does.
- Proof: replaces the proof above. The full routine Rust suite passes with `BIOMCP_CACHE_MIN_DISK_FREE=100%` exported. The five tests fail that way on current main.
- Defers: moving `StaleNoteEnv` onto `FixtureEnv`.
