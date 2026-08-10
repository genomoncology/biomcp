---
flow: build
priority: 9
deps: ["0934"]
---
# Keep normal builds from rewriting generated source

An ordinary AlphaGenome-enabled build runs the locally installed `protoc` and
can overwrite the tracked generated Rust fallback when its bytes differ. The
resulting executable can therefore contain code absent from the stamped commit
while merely building dirties the checkout.

## Build contract

`cargo check`, `cargo build`, `cargo test`, packaging, and release builds write
only to Cargo output directories and explicitly declared temporary locations.
They consume the committed AlphaGenome generated Rust bytes and never modify a
tracked path. The absence, version, or output of a workstation `protoc` does
not change normal build inputs.

Provide one explicit maintainer regeneration command. It uses the repository's
pinned `protoc` version, writes a temporary candidate, applies the deliberate
dead-code annotation transformation, compares it with the tracked file, and
replaces that one file only after successful generation and validation. A
check-only mode fails when regeneration would differ without editing the tree.

## Done when

- Normal builds with no `protoc`, the pinned `protoc`, and a controlled
  differing generator all leave `git diff --exit-code` clean and consume the
  committed bytes.
- The explicit regeneration command changes only the expected generated file,
  and check-only mode reports the same diff without writing.
- A binary's embedded commit identity corresponds to all source bytes used to
  build it; no generated content is refreshed after identity is resolved.
- CI runs regeneration check with the pinned tool but normal developer builds
  do not require `protoc`.
- Troubleshooting, contributor, architecture, and release documentation state
  the same behavior.

## Authorized test changes

Design commits may restate `build.rs`, add a regeneration script and build
provenance tests, adjust pinned protoc setup, and correct related documentation.
The committed generated API surface and AlphaGenome behavior remain covered;
no unrelated generated formatting churn belongs here.

The src line ceiling may not rise except for a reviewed change in the generated
file produced by the explicit command.
