---
flow: build
priority: 10
deps: ["0951"]
---
# Isolate build identity from reusable Rust compilation

`build.rs` watches Git HEAD and exports commit, version, and date values to the
whole BioMCP package. Every design, review, and code commit can therefore
invalidate compilation of the roughly 196,000-line main crate merely to update
version output.

## Test contract

Build once, move HEAD with a metadata-only commit, and rebuild under captured
Cargo diagnostics. Prove the reusable library artifacts remain fresh while the
smallest identity-owning executable unit is rebuilt and reports the new commit.

## Done when

- Commit identity remains truthful for version text and JSON, tagged releases,
  archives without Git metadata, and dirty developer worktrees.
- Moving only HEAD does not recompile the reusable product library or its unit
  tests.
- Changing an identity consumer rebuilds that consumer; changing product source
  rebuilds the normal affected artifacts.
- Reproducible package builds do not depend on an arbitrary surrounding Git
  checkout.
- Cold, warm, and HEAD-only rebuild timings and rebuilt crate lists are recorded.

## Authorized test changes

Design commits may split the package into a reusable library and thin binary
identity owner, restate `build.rs`, and adjust version/build-provenance tests.
Public commands, version fields, and release identity semantics do not change.

The src line ceiling may rise by at most 80 lines.
