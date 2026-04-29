# CLI Module Decomposition Contract

## Why this exists

BioMCP already established a practical CLI architecture cap: no single runtime or
sidecar test file under `src/cli/` should grow beyond 700 lines. Tickets 180,
181, and 185 proved the pattern in `src/cli/mod.rs` and the entity-family
subdirectories, but the rule was never written down as a reusable technical
contract. Large flat files then regrew around the facade.

This document is the durable pattern future CLI refactors must follow.
`architecture/technical/cli-decomposition-2026.md` is the current migration
plan; this document defines the steady-state rules.

## Current problems to avoid

Oversized CLI files fail for the same recurring reasons:

1. **One file owns multiple responsibility zones.** Planning, dispatch,
   formatting, filesystem helpers, and tests end up interleaved.
2. **Inline or flat test ownership hides the runtime seam.** Tests inflate file
   size and make it hard to tell what the runtime contract actually is.
3. **The public module path becomes coupled to the implementation layout.**
   Internal moves feel risky because downstream imports depend on a flat file.
4. **The 700-line cap is treated as a convention rather than a ratchet.**
   Without an executable check, files drift back over the limit.

## Target state

Every CLI area that grows beyond one review-sized file should follow the same
shape:

```text
src/cli/<area>/
  mod.rs            # stable facade: public types, public entry points, re-exports
  <zone>.rs         # one ownership zone per file
  <zone>.rs
  <zone>/tests.rs   # sidecar tests when a zone has enough private logic
  tests.rs          # area-wide tests only when one file is enough
```

The file that used to be `src/cli/<area>.rs` becomes either:

- `src/cli/<area>/mod.rs`, or
- a tiny forwarding facade that immediately re-exports from `src/cli/<area>/`.

Prefer the directory form when the area owns multiple internal files.

## Facade rules

`src/cli/<area>/mod.rs` is the stable module boundary.

It may contain:

- the `//!` module doc comment
- public structs/enums/type aliases that define the stable in-crate API
- public entry points used by `src/cli/outcome.rs`, `src/mcp/shell.rs`, or
  renderers
- `mod` declarations and intentionally-scoped `pub(crate)` / `pub(super)`
  re-exports
- `#[cfg(test)] mod tests;` declarations when the area needs an area-wide test
  sidecar

It should not become a new catch-all. Planning, formatting, transport,
filesystem, and test helper logic belong in sibling files.

## Responsibility splitting rules

Split by ownership zone, not by arbitrary line count. Good zones are:

- **catalog / static data** — command catalogs, source descriptors, route tables
- **planning** — input normalization, route selection, dispatch plans
- **execution / transport** — async calls, subprocesses, timeout handling
- **formatting / rendering helpers** — markdown fragments, JSON shaping,
  follow-up command generation
- **local runtime helpers** — filesystem probing, install paths, cache helpers
- **test support** — fixtures or helpers shared by multiple sidecars

The usual target is **2-4 runtime files per area**. If an area still needs more,
prefer a nested directory for one heavy subdomain rather than a new flat
1,000-line sibling file.

## Test ownership rules

- Runtime logic should use sidecar tests next to the owning file whenever those
  tests need private helper access.
- Large flat test files should themselves become directories, for example
  `src/cli/article/tests/{help,json,filters}.rs`.
- Shared test helpers belong in the narrowest owning module. Only lift them into
  `src/cli/test_support.rs` or `src/test_support.rs` when more than one area
  genuinely shares them.
- Inline `mod tests { ... }` blocks are acceptable only for tiny private helper
  checks that would become more awkward as a sibling file. They are not the
  default decomposition pattern.

## Public-surface invariants

CLI decomposition tickets are behavior-preserving refactors. They must preserve:

1. **Top-level command grammar.** No clap shape changes, no help-text drift, no
   added or removed commands.
2. **Stable module paths used elsewhere in the crate.** Keep paths like
   `crate::cli::search_all::SearchAllResults` stable via facade-owned types and
   re-exports.
3. **Existing markdown/JSON contracts.** Specs, contract tests, and renderers
   must stay green without downstream doc rewrites.
4. **MCP boundaries.** Refactors must not widen CLI-only commands onto the MCP
   surface or change resource behavior.

For any decomposition slice, proof should include:

- `cargo test <focused cli filter> --lib`
- `cargo clippy --lib --tests -- -D warnings`
- `make spec-pr`
- the existing help/contract canaries for the affected surface (for example
  `spec/surface/cli.md`, `spec/surface/discover.md`, or render-long-help unit
  tests)

## Ratchet rules

`make check` runs `tools/check-quality-ratchet.py`, which performs a global scan
of tracked Rust files under `src/cli/` and enforces the 700-line cap. Files over
the cap must either be decomposed or appear in `tools/cli-line-cap-allowlist.json`
with a dated follow-up ticket reference and their current line count. The
allowlist is a ratchet: entries become failures once the file is under the cap,
and allowlisted files may not grow beyond the recorded count.

Every decomposed CLI area must also add an executable structure ratchet in a dedicated
integration test, normally named:

- `tests/<area>_cli_structure.rs`, or
- `tests/<area>_structure.rs` when the area name is already unambiguous.

That ratchet should assert all of the following for the new directory:

1. every tracked Rust file under the decomposed area starts with a `//!` header
2. every tracked Rust file under the decomposed area stays at or below 700 lines
3. the expected runtime/test layout exists, without placeholder modules left
   behind

Using one ratchet file per area keeps decomposition tickets independently
mergeable and avoids a shared mega-ratchet file that every refactor has to edit.

## Migration playbook

When decomposing a large CLI file:

1. identify the stable public entry points and downstream import paths
2. identify 2-4 responsibility zones
3. create `src/cli/<area>/mod.rs` as the stable facade
4. move one zone at a time into sibling files, keeping the crate path stable via
   facade re-exports
5. move tests beside the runtime code they exercise
6. add the area-specific structure ratchet test
7. prove the public surface is unchanged with focused Rust tests plus
   `make spec-pr`

## Reference patterns

Existing BioMCP examples to follow:

- `src/cli/mod.rs` — thin facade over focused siblings
- `src/cli/article/` — command-family directory with runtime split and sidecar
  tests
- `src/entities/article/` and `src/entities/variant/` — stable facade + zone
  files + sidecar tests + structure ratchet precedent

Use this document as the reusable contract; use
`architecture/technical/cli-decomposition-2026.md` for the current oversized CLI
migration sequence.
