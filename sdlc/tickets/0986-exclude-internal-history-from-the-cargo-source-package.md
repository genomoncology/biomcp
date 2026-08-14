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

Verification must exercise the package, not only list it. Construct the
`.crate` with `cargo package --allow-dirty --locked`; Cargo's built-in package
verification must compile the normalized crate and its normal embedded runtime
assets. Then unpack that generated archive into a temporary directory outside
the source checkout and compile one focused, package-safe integration test from
the extracted manifest. That test must link the packaged library and check a
public, deterministic build-identity value.

Do not require the entire extracted unit/integration test graph to compile.
Many repository-only tests intentionally use excluded `testdata/` fixtures,
and the MCP contract test uses a path-only development dependency that Cargo
does not publish. Making those tests redistributable is separate work. Instead,
statically reject packaged Rust `include_str!` or `include_bytes!` references
into `architecture/` or `sdlc/`, and prove the moved alias assertion in its
normal repository test.

## Done when

- `cargo package --list` contains no `sdlc/` or `architecture/` path and no more than 1,300 files.
- `cargo package --allow-dirty --locked` succeeds, and the resulting archive
  compiles the focused package-safe integration target after extraction
  outside the source checkout.
- Packaged Rust sources have no compile-time include path into `architecture/`
  or `sdlc/`; full fixture-backed repository test compilation is explicitly
  outside this ticket.
- Runtime embedded assets, public CLI documentation alignment checks, and
  ordinary source-package checks remain intact.

## Authorized test changes

The design may restate `tests/test_source_package_boundary.py` and
`src/cli/drug/alias_alignment_tests.rs`, and may add the focused package-safe
integration target under `tests/`.
