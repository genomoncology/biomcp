---
base: 3b0c8fcf
head: 76633b95
---

Made the cache-expiry test deterministic.

`cache::migration::tests::async_io_crossing_expiry_settles_without_admitting_a_mutation`
paused the clock, released the operation, and then read a real file. `timeout_at`
polls the wrapped future before the timer, so whenever the read resolved before
the join-handle poll the operation beat the expired timer, `unwrap_err()` saw an
`Ok`, and the test failed once per long run. The operation is now an in-memory
completion that yields once, which guarantees it is pending on the poll that
checks the timer, and the entry select is biased on the injected pause. The
edit is line-neutral, so `src/cache/migration.rs` stays at its pinned 1086-line
baseline.

Evidence: a temporary red-repro test failed with
`called Result::unwrap_err() on an Ok value: ()`, the same panic the recorded
flake produced; `cargo fmt --check`, `cargo clippy --locked --no-default-features
--all-targets -- -D warnings`, and the quality ratchet pass; the isolated test
passes; three consecutive full nextest runs pass 3756/3756; `make lint` and
`make test` on yellow at 76633b95 pass (3756 Rust, 964 Python, 3 skipped).

Reviews: the design review rejected the first draft for an unbounded acceptance
criterion; the code review accepted and confirmed the race is removed rather
than narrowed. The refusal evidence after the change is the `Err(TimedOut)`
assertion at `src/cache/migration.rs:889`; the `untouched` and epoch checks stay
but the in-memory operation never touches them. Real async-IO deadline coverage
survives in `epoch_cleanup_stops_mutating_after_a_mid_traversal_deadline`.
