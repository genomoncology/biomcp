---
flow: build
priority: 6
---
# Exclude internal history from the Cargo source package

The Cargo source package currently contains 2,629 files, including the full
SDLC ledger and architecture experiment history. Those files are not required
to build or use the crate and make the published artifact needlessly large.

Exclude `sdlc/` and `architecture/` from the Cargo package. Keep runtime
assets, public documentation, examples, specifications, tests, skills, and
templates that remain inside the deliberate package boundary. Ratchet the
inventory so internal history cannot return unnoticed.

## Done when

- `cargo package --list` contains no `sdlc/` or `architecture/` path and no more than 1,300 files.
- The packaged crate verifies and builds from its extracted package without using the source checkout.
- Runtime embedded assets and ordinary source-package checks remain intact.

## Authorized test changes

The design may restate `tests/test_source_package_boundary.py` and package
construction assertions that consume `Cargo.toml`.
