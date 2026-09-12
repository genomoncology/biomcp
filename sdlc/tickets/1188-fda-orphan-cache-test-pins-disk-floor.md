---
flow: build
priority: 1
deps: []
---

# 1188: The FDA orphan cache freshness test pins its disk floor

## Goal

`sources::tests::provider_network::fda_orphan_normal_cache_is_fresh_then_refreshes_when_expired`
sets `BIOMCP_CACHE_MIN_DISK_FREE=1B` alongside its existing environment
restores, so the fixture no longer depends on the host's free disk.

## Current Facts

The test sets only `BIOMCP_FDA_ORPHAN_BASE` and `BIOMCP_CACHE_DIR`, so it
inherits the production default `DiskFreeThreshold::Percent(10)`
(`src/cache/config.rs:12`). On a host with less than ten percent free, cache
eviction removes the test's cached entry before
`rewrite_cached_time_for_test` runs, and the exact test fails on otherwise
clean main. A downstream migration team hit this on a host at roughly 8.9
percent free and reported it as blocking their cutover. Reproduced
deterministically on 2026-09-12 on clean main at `193cfd6e` by running the
exact test with `BIOMCP_CACHE_MIN_DISK_FREE=100%`: failed in 0.216 s with
eviction forced. `parse_disk_free_threshold` accepts `1B` as one byte, the
smallest legal value (`src/cache/config.rs:360-381`). Design review traced the
eviction end to end: a violated floor collapses the effective size target on
the put path and the planner removes oldest-first regardless of entry age, so
the just-written fresh entry is evicted; with the fix the pin overrides the
floor before the first fetch resolves config.

## Scope

Add `("BIOMCP_CACHE_MIN_DISK_FREE", Some("1B"))` to the test's existing
`EnvRestore::set` list in `src/sources/tests/provider_network.rs`. No
production code changes; no other tests change.

## Acceptance

The exact test passes with `BIOMCP_CACHE_MIN_DISK_FREE=100%` forced in the
environment (the override proves host independence) and passes normally.
`make lint` and the focused provider-network suite pass; full gates run on
the gate host before merge to main. The package path count stays 1,300.

## Dependencies

None.

## Complexity

- Contract score: 0 (one exact existing rule: the environment override exists)
- State and timing score: 0 (pure local test behavior)
- Reach score: 0 (one test function)
- Proof score: 0 (one focused deterministic check, both directions)
- Cost of error score: 0 (test-only, cheap local correction)
- Total: 0
- Minimum level floor: none
- Final level: 1
- Reasons: single-site test-independence fix with an existing override
- Selected model: gpt-5.6-luna, high reasoning (level 1 implementer)

## Review

- Design review: pending
- Code review: pending
