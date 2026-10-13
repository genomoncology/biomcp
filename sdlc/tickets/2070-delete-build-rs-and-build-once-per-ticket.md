# 2070 — Delete build.rs and build once per ticket

Status: OPEN.

Milestone: 0.9.3

## Outcome

Editing a doc, a spec page or a Python test recompiles nothing, and a ticket's gates compile BioMCP once in the dev profile.

## Evidence

- An empty `build.rs` makes cargo rescan the package and recompile the crate after any file change: a spec page touch costs 175 s, and 2.7 s with the file gone (measured, `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 1).
- `make test` and `make spec` build a second `spec` profile binary on top of the dev build, 253 s after a Rust edit, though both lanes pass on the dev binary (step 2).

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, steps 1 and 2.
- Keeps: the release profile for `make release-gate` and the release workflow.
- Changes: delete `build.rs`; point the Python and spec lanes at the dev test build's binary; drop the `spec` profile from routine gates; set `CARGO_INCREMENTAL=1` in the documented targeted-test loop only.
- Proof: on the build host, a spec-page touch then `cargo build --all-targets` compiles nothing; `make test` and `make spec` after a Rust edit print one `Compiling biomcp-cli`; their wall times recorded before and after; a CI run that touches only a spec page compiles nothing.
- Defers: CI job structure (2075).
