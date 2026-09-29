# Cache mode is fixed for the whole test process

Filed 2026-09-27 from GitHub #286: https://github.com/genomoncology/biomcp/issues/286. Status: fixed on main (ticket 1261, gate 8c37f045). Close #286 when a release ships it.

## Symptom

On 0.9.0, `cargo test --lib -- --test-threads=1` reports 3497 passed, 8 failed and 26 ignored. The failures include:

- `cli::system::batch::tests::dropping_settlement_cancels_cacheable_article_provider_retry_and_admission`
- `entities::article::graph::tests::cache::a_corrupt_record_is_a_miss_and_is_replaced`
- `entities::article::graph::tests::cache::a_future_schema_version_is_a_miss`
- `entities::article::graph::tests::cache::repeat_call_serves_the_stored_edge_without_graph_or_fulltext_requests`

Each failing test passes alone. The result depends on test order, not on thread count.

## Cause

`env_cache_mode()` in `src/sources/mod.rs:394` stores the first `BIOMCP_CACHE_MODE` value in a process-wide `OnceLock`. Tests in `src/cli/discover.rs:293` and `src/entities/drug/test_support.rs:149` set the variable to `off` and restore it afterwards. Restoring the variable cannot reset the `OnceLock`, so every later test in the binary runs with the cache bypassed. The code is unchanged on main at 9148e874. `cache_infinite` at `src/sources/mod.rs:442` reads the variable on each call, so the two cache-mode readers already disagree about when a change takes effect.

## Fix

The process should read the mode once, and tests should be able to set it without leaking into other tests. Two options:

1. Read the variable on each call, as `cache_infinite` does. The cost is one environment read per request.
2. Keep the `OnceLock` for the process. Add a test-only override that the tests set and clear in a scoped guard, like the existing `NO_CACHE` task-local.

Option 2 keeps the release behaviour unchanged.

## Success criteria

- `cargo test --lib`, with and without `--test-threads=1`, passes in any test order.
- A regression test sets the mode to `off` and then runs a cache read in the same binary. The read hits the cache.
