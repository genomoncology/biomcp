# Shrink the large futures and share the runtime

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-large-futures-and-per-call-runtimes.md`.

## Problem

The top-level command future reaches 145 KB and the 8 MiB execute stack
left only about sixteen times headroom in release (less in debug)
before ticket 1225's fix; the cause is `with_no_cache` taking a future
by value (doubling 71 KB to 142 KB), unboxed command dispatch futures,
and an inlined `tokio::join!` in both match arms of the discover chain.
Separately, every CLI call builds and drops a Tokio runtime; dropping
waits unboundedly for background blocking work, so a timed-out call
still blocks the reply, and pooled connections stay tied to dead
runtimes. XML parsing caps node count but not nesting depth, so a
deeply nested document on a 2 MiB blocking thread can overflow.

## Design

1. Make `with_no_cache` a plain function returning
   `NO_CACHE.scope(no_cache, fut)` so the future is taken once.
2. `Box::pin` each command dispatch future in the outcome runner; box
   the discover join.
3. Add a test that fails when the top-level future passes a set size
   (type-name size probe, like the nightly type-size build).
4. Keep one long-lived runtime per server process; CLI one-shots use
   `shutdown_background` so drop never blocks on background work.
5. Cap XML nesting depth and raise the blocking pool's stack size; a
   depth-overflowing document returns an error, not a crash.

## Acceptance

- The size-probe test exists and the measured top-level future is
  recorded before and after.
- No per-call runtime drop blocks the reply (test with a hanging
  background task).
- The XML depth cap rejects a nesting bomb with an error.

## Review

- Design review: pending
- Code review: pending
