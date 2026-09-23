# Make the MCP server survive a panicking tool call

Filed from `sdlc/issues/2026-09-23-a-panic-in-one-tool-call-ends-the-mcp-server.md`. Blocks 0.9.1.

## Problem

The release profile sets `panic = "abort"` (`Cargo.toml:150`). Two places in
the code expect to recover from a panic:

- `src/cli/outcome.rs:616` turns a failed execute-thread join into a
  "worker panicked" error result.
- `src/entities/article/graph/citation_evidence.rs:547` recovers from a
  panicking traversal.

Under abort, neither runs. Any panic ends the process, and a long-running MCP
server loses every session. Tests run in debug builds, where panics unwind, so
the isolation is tested but never effective in production.

One reachable trigger is real today: `src/sources/clingen.rs:638` slices an
external date with `&value[..10]`, which panics when byte 10 falls inside a
multi-byte character.

## Design

1. Set `panic = "unwind"` in `[profile.release]`. Before landing, find why
   abort was set (git history on `Cargo.toml`) and measure the binary-size
   cost on the gate host; record both.
2. Replace the ClinGen slice with a bounds-checked `value.get(..10)` that
   returns `None` (the function already returns `Option<String>`).
3. Sweep `src/` (not tests) for other byte-index slicing on strings built
   from external data (`[..N]`, `[a..b]`, `split_at`). Fix the genuine cases
   the same way; enumerate in this ticket anything left alone and why.
4. Add a test that a panicking execute worker yields an error result and the
   process survives, and run that test in the release profile too. The yellow
   gate adds a focused `cargo nextest run --release` invocation for it.
5. Confirm by citation that MCP tool dispatch rides the same execute seam in
   `src/cli/outcome.rs`, so the CLI and the server both regain isolation.

## Acceptance

- `[profile.release]` has `panic = "unwind"`; the ticket records the origin
  of abort and the measured `.text` delta on yellow.
- The ClinGen slice is gone; the sweep's findings are enumerated here.
- The panic-isolation test passes in both debug and release profiles on
  yellow; the focused release run is part of the gate evidence.
- Full yellow gate at the head SHA: `make lint`, `make test`, `make spec`.
- A record lands in `sdlc/records/` and the issue file gains a Resolved
  section.

## Review

- Design review: pending
- Code review: pending
