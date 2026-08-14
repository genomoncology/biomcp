---
flow: build
priority: 6
---
# Exclude internal history from the Cargo source package

The Cargo source package currently contains more than 2,600 files, including
the full SDLC ledger and architecture experiment history. Those files are not
required to build or use the crate and make a future or manually constructed
crates.io source package needlessly large. BioMCP's current GitHub archives and
PyPI wheels use separate packaging paths; this ticket does not claim to shrink
those shipped artifacts or add crates.io publication.

Exclude `sdlc/` and `architecture/` from the Cargo package. Keep runtime
assets, public documentation, examples, specifications, tests, skills, and
templates that remain inside the deliberate package boundary. Ratchet the
inventory so internal history cannot return unnoticed.

One packaged Rust test currently reads
`architecture/ux/cli-reference.md` with `include_str!`. Move that assertion to
the equivalent shipped public command documentation so excluding architecture
does not leave the packaged test graph uncompilable. Do not retain a private
architecture file merely to satisfy a test.

Verification must exercise the package, not only list it. Construct and verify
the `.crate` with `cargo package --allow-dirty --locked`, unpack that generated
archive into a temporary directory outside the source checkout, then run a
locked build and `cargo test --no-run` using only the extracted manifest. The
isolated check must fail on a checkout path dependency, missing embedded asset,
or packaged test that still references an excluded file.

## Done when

- `cargo package --list` contains no `sdlc/` or `architecture/` path and no more than 1,300 files.
- `cargo package --allow-dirty --locked` succeeds, and the resulting archive
  builds and compiles its tests after extraction outside the source checkout.
- Runtime embedded assets, public CLI documentation alignment checks, and
  ordinary source-package checks remain intact.

## Authorized test changes

The design may restate `tests/test_source_package_boundary.py` and
`src/cli/drug/alias_alignment_tests.rs`.
