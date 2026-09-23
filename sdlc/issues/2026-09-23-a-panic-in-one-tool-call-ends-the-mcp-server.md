# A panic in one tool call ends the MCP server

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Blocks 0.9.1.

## Symptom

The release profile sets `panic = "abort"` (`Cargo.toml:150`). Two places expect to recover from a panic:

- `src/cli/outcome.rs:616` turns a failed thread join into "worker panicked".
- `src/entities/article/graph/citation_evidence.rs:547`.

Under abort, neither recovery runs. Any panic ends the process, and a long-running MCP server loses every session. Tests run in debug builds, where panics unwind, so they pass.

One reachable trigger: `src/sources/clingen.rs:638` slices an external date with `&value[..10]`. The slice panics when byte 10 falls inside a multi-byte character.

## Fix

- Use `panic = "unwind"` in the release profile. The existing thread join already isolates failures.
- Replace the ClinGen slice with `value.get(..10)?`.
- Search for other byte-index slices on external strings and fix them the same way.
- Add a release-profile test that a panicking worker yields an error result and the process survives.
