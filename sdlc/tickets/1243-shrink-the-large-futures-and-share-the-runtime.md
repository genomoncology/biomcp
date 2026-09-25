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
3. A stable size probe: a unit test in `src/cli/outcome/tests.rs`
   parses a minimal CLI, constructs the future, and asserts
   `std::mem::size_of_val(&fut)` under a recorded ceiling (future
   construction is lazy; no I/O runs). The probe bounds
   `run_outcome_inner`'s future and the `run` fallthrough directly —
   not `run_outcome`'s wrapper, which is already boxed and tiny.
   Record the measured before/after in the ticket.
4. Stop constructing a runtime per call. The construction site is
   `run_outcome_with_worker_stack` (`src/cli/outcome.rs:610-626`),
   and it serves both process kinds: CLI one-shots via `run_outcome`
   and every MCP tool call via `mcp/shell.rs:414`. Split the paths:
   the MCP path drives the command future on the shared server
   runtime (capture the `Handle`, `block_on` from the dedicated
   8 MiB thread — `EXECUTE_STACK_BYTES` stays 8 MiB per ticket
   1225's pin), so a server process keeps one long-lived runtime and
   pooled connections stop dying per call; the CLI one-shot keeps a
   per-call runtime but drops it with `shutdown_background`, so the
   reply never blocks on background work. Background work today is
   only cache eviction (`spawn_eviction_task`,
   `src/cache/manager.rs:407-431`); all cache puts are awaited inline
   (`manager.rs:222-289`), and that invariant is what makes
   `shutdown_background` safe — record it in the code comment, since
   fire-and-forget puts would be lost under `shutdown_background` +
   `process::exit`. Acceptance: a test that MCP tool dispatch
   constructs no runtime (assert the construction path is not taken —
   e.g. a counter or a stub seam), and the hanging-background-task
   test covers a blocking task (a started eviction-style task must
   not delay the reply).
5. XML depth cap: a post-parse iterative walk inside
   `parse_external_xml` (`src/xml.rs:13-85`) counting element depth,
   rejecting above the cap with a new `ExternalXmlError` variant; the
   existing byte loop ignores tag structure and is the wrong place.
   The walk protects the recursive JATS/ClinVar walkers on the 2 MiB
   blocking threads; the acceptance nesting-bomb test also proves
   roxmltree's own parse does not recurse by depth. The blocking-pool
   stack raise names its runtimes: the `outcome.rs:615` builder for
   CLI one-shots, and the shared server runtime (replace
   `#[tokio::main]` with an explicit `Builder`) — headroom on top of
   the depth cap, which is the control.
6. Deferred from the issue, recorded so closing the umbrella does not
   erase them: retries firing on POST requests, and
   `TRIAL_ALIAS_CACHE` growing without limit
   (`src/entities/drug/get.rs:353`).

## Acceptance

- The size-probe test exists and the measured top-level future is
  recorded before and after.
- No per-call runtime drop blocks the reply (test with a hanging
  background task).
- The XML depth cap rejects a nesting bomb with an error.

## Review

- Design review: REJECT once (item 4 named the wrong seam — the
  per-call runtime at outcome.rs:610-626 serves the MCP path too;
  probe misstated; depth-cap placement unspecified; two issue items
  dropped silently), findings folded, re-review pending
- Code review: pending
