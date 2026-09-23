---
flow: build
priority: 4
deps: []
---

# 1228: Make the cache-expiry test deterministic

## Goal

The cache-expiry test stops racing real file IO against a paused clock, so it either fails every time or passes every time instead of failing once per long run.

## Current Facts

- `cache::migration::tests::async_io_crossing_expiry_settles_without_admitting_a_mutation` (`src/cache/migration.rs:862-889`) failed once in a full nextest run, passed on rerun, and passed 20/20 in isolation (record 1219).
- `deadline_io` accepts any `impl Future<Output = io::Result<T>>` (`src/cache/migration.rs:205-221`), so an in-memory completion can replace the real `tokio::fs::read` at `:879`.
- The flake window is tokio's inner-first timeout poll in `timeout_at` plus a blocking read that can resolve before the join-handle poll; the non-biased `select` at `:883-886` and the paused-clock auto-advance widen it, and `:887-889` is the only explicit clock control.
- `VariantArticleDeadline::run` is `timeout_at`, which polls the operation first (`src/sources/mod.rs:80-87`), so an operation that stays pending hangs the test and one that is ready on the first poll can beat the expired timer.

## Design

- Reproduce the failure first with a starvation simulation: an operation that is ready on its first poll races the expired `timeout_at` timer (`src/sources/mod.rs:80-87`) and the old assertion fails with an unexpected `Ok`. That is the observable the red repro must show, and the red output stays in the record.
- Keep the `entered` handshake outside the deadline-wrapped future, or bias the select on `entered`, and replace the real file read with an in-memory completion (a `Notify` or `oneshot`) that stays pending on the poll where the timer is checked, then yields and completes.
- Removing the real read also removes the only real async-IO crossing, so say which assertion still carries the refusal evidence: the `Err(TimedOut)` assertion at `src/cache/migration.rs:889` carries it, and the record must say so; the `untouched` marker and the epoch check at `:890-891` stay, but the in-memory operation never touches them. Real async-IO deadline coverage survives in `epoch_cleanup_stops_mutating_after_a_mid_traversal_deadline`.
- Do not change `deadline_io`'s production semantics or any code outside the test module.

## Acceptance

1. The deterministic red repro shows the failure mode the old test could hit.
2. The modified test passes in three consecutive full nextest runs on yellow.
3. `src/cache/migration.rs` is unchanged outside the test module.
4. `make lint` and `make test` pass on yellow at the pushed SHA, and CI `canonical-gates` is green on main.

## Out of scope

- Cache-migration behavior changes.
- Other flaky tests.

## Complexity

- Level 2 (contract 0, state and timing 2, reach 0, proof 1, cost of error 1 = 4)
- Reasons: paused clock, select ordering, and a timer race are the whole problem; the change is confined to one test module and a wrong fix hides the ordering hazard.

## Review

- Design review: REJECT 2026-09-22 (gpt-5.6-sol, medium) — the first draft's acceptance criterion had no bound; the rewrite named the red repro and three consecutive runs
- Code review: ACCEPT 2026-09-22 (gpt-5.6-sol, medium) — confirmed the race is removed rather than narrowed; the record names `src/cache/migration.rs:889` as the refusal evidence
- Verification: red repro panicked with `unwrap_err()` on an `Ok`, fmt and clippy clean, isolated test passes, three consecutive full nextest runs 3756/3756, yellow `make lint` and `make test` OK at 76633b95; see `sdlc/records/1228-make-the-cache-expiry-test-deterministic.md`
