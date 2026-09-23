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
3. Fix four more genuine external-string slicing sites found in design
   review, and enumerate the safe-to-leave list in this ticket:
   - `src/transform/drug.rs:246-260`: a 10-byte string is accepted
     unvalidated, then sliced `[0..4]/[5..7]/[8..10]` by callers at
     `:280` and `:291` on data from external drug JSON.
   - `src/sources/europepmc.rs:382`: `split_at(3.min(len))` panics when a
     multibyte character straddles byte 3 of a tool-supplied PMCID.
   - `src/cli/benchmark/run/execute.rs:310`: `&compact[..240]` on joined
     benchmark output.
   - Safe to leave (guarded ASCII, u8 slices with length checks, or
     boundary-checked): `utils/date.rs:47-67`, `sources/kegg.rs:266-271`,
     `entities/drug/metadata.rs:113-118`, `entities/gene.rs:983-990`,
     `entities/trial/get.rs:192-198`, `entities/author/mod.rs:183-188`,
     `cache/*`, `gencc/store.rs`, `sources/rate_limit.rs:41-46`,
     `provider_url_policy.rs:608-610`.
4. Update `tests/test_upstream_planning_analysis_docs.py:1183`, which pins
   the exact `[profile.release]` block including `panic = "abort"`. This
   pin is the durable enforcement against a regression to abort: cargo
   forces unwind on test targets, so no Rust test can detect the profile
   flipping back.
5. Add a test that a panicking execute worker yields an error result and the
   process survives, and run that test in the release profile too. The yellow
   gate adds a focused `cargo nextest run --release` invocation for it.
6. Confirm by citation that MCP tool dispatch rides the same execute seam in
   `src/cli/outcome.rs` (design review cited `src/mcp/shell.rs:360` →
   `execute_mcp_cli` → `run_outcome_with_worker_stack`), so the CLI and the
   server both regain isolation.

Origin note for the record: `panic = "abort"` arrived in `09b789d2`
("Rust 080 (#124)", 2026-02-23) with no recorded rationale, and the 0.9
runbook shows abort caused gate-host SIGABRT flakes. Under unwind, the
wheel-smoke 134 check goes dormant for panics (a panic now exits 101, which
still fails the smoke's success requirement for the swept commands).

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

- Design review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — P1 additions folded
  in (the profile-pin test must flip with Cargo.toml; three more genuine
  slice sites; the pin is the real abort-regression enforcement)
- Code review: ACCEPT 2026-09-23 (gpt-5.6-sol, medium) — all six fix sites
  verified; P2 notes: the panic test mirrors the seam rather than calling
  it, and the DrugCentral guard tightens 10-byte non-date values
- Verification: yellow gate at 6f3dffb3 lint/test/spec OK; focused
  release-profile `nextest -E 'test(worker_panic)'` OK; unwind size cost
  measured (text +5,557,831, file +5,559,048, about +17 percent); see
  `sdlc/records/1230-make-the-mcp-server-survive-a-panicking-tool-call.md`
