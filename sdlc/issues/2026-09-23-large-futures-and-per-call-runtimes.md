# Large futures and per-call runtimes

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Follows the stack overflow RCA in `2026-09-17-pypi-wheel-binary-stack-overflows-on-trial-search.md`.

## Stack depth

Measured with gdb on a current release build, peak command-thread stack use against the 8 MiB limit:

- trial search: about 404 KB
- drug trials: 428 KB
- drug interactions: 420 KB
- `search all`: 520 KB

Release builds have about sixteen times headroom. Debug builds use more than sixteen times as much stack. A nightly type-size build shows the cause:

- `with_no_cache` (`src/sources/mod.rs:383`) is an `async fn` that takes a future by value. It keeps the argument and the wrapper, doubling the size: 71,376 bytes becomes 142,784 at `src/cli/outcome.rs:476`.
- The top-level command future is 145 KB. The drug to trial-name to discover chain nests a future of about 50 KB through about fifteen levels. `src/entities/discover.rs:482-499` inlines a `tokio::join!` in both match arms.

Fix: make `with_no_cache` a plain function returning `NO_CACHE.scope(no_cache, fut)`. `Box::pin` each command dispatch future in `run_outcome_inner`. Box the discover join. Add a test that fails when the top-level future passes a set size.

## A new runtime per call

`src/cli/outcome.rs:604` builds a Tokio runtime for every call. Dropping it waits with no limit for background blocking work, so a timed-out parse at `citation_evidence.rs:541` still blocks the reply. The shared HTTP client (`src/sources/mod.rs:344`) keeps pooled connections tied to runtimes that have ended. Keep one long-lived runtime per server, or use `shutdown_background`.

## XML nesting depth

`src/xml.rs` caps node count but not nesting depth. Full-text parsing runs on blocking threads at the 2 MiB default stack. A deeply nested document can overflow and end the server. Cap depth and set `thread_stack_size`.

## Smaller items

- Retries fire on POST requests.
- `TRIAL_ALIAS_CACHE` grows without limit.
