---
flow: build
priority: 3
deps: ["0669", "0671", "0672", "0674", "0677", "0682", "0684", "0892", "0901", "0902", "0903", "0904", "0905", "0906", "0907", "0913", "0919", "0920"]
---
# Reconcile the final live-spec registry after source conversions

This is the single final registry owner. Superseded ticket 0885 must not
rebuild a competing inventory.

## Done when

After every dependency lands, BioMCP has one truthful answer to which specs
are routine and which are live:

- scripts/run-specs.sh is the canonical registry.
- Makefile targets ask the runner for the registry instead of keeping a
  second hand-written list.
- The runner's declaration and verification paths consume the same data.
- Architecture documentation is checked against the canonical list rather
  than carrying an unchecked count.
- The separate NIH disease/gene lane is either represented in that source of
  truth or explicitly named as a distinct lane with no duplicate execution.

The current 16-, 17-, and 18-path answers must collapse to one exact set.

## Work

Re-inventory the repository at flight time. For every path:

1. identify its owning conversion record;
2. prove its routine blocks cover CLI parsing, request construction, response
   decoding, and rendering;
3. leave genuinely mutable provider availability only in the live verify
   lane;
4. classify stale files such as ddinter-live.md from evidence, not filename.

Convert the remaining mixed blocks in spec/surface/cli.md only when their
source owner has landed. Do not reimplement article graph/assets,
MyVariant/hotspots, or variant-article conversion work already owned by
completed records or 0684.

## Authorized test changes

Design commits may restate spec/surface/cli.md, scripts/run-specs.sh,
Makefile, architecture/technical/live-spec-conversion-target.md, and tests
that assert the registry. No biomedical behavior, CLI grammar, or provider
code changes belong here.

The src line ceiling may not rise.
