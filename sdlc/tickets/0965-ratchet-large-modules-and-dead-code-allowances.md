---
flow: build
priority: 5
deps: ["0957"]
---
# Ratchet large modules and dead-code allowances

BioMCP has several multi-thousand-line entity/source modules and broad
`allow(dead_code)` attributes. Do measured maintenance: stop further growth,
make every exception accountable, and remove duplicate fixture-only projections
without launching a repository-wide rewrite.

## Ratchet contract

Generate and commit a deterministic inventory from tracked `src/**/*.rs` files.
For every existing file over 1,000 physical lines, record its exact baseline and
reject growth above it. A new Rust source file may not start over 1,000 lines.
Files at or below the threshold may grow up to it. The inventory update command
is explicit, never runs during ordinary build, and CI rejects an unexplained
raised baseline. A ticket may authorize one named increase only by recording
the file, exact delta, reason, and removal condition in the ratchet data.

Inventory every `allow(dead_code)` by file/item. Reject new whole-module or
whole-file allowances. Existing broad allowances move to a checked exception
list with an item owner, concrete runtime/test reason, and removal condition;
otherwise narrow or remove them. New item-level allowance requires the same
fields and an adjacent comment. Generated bindings are excluded only by exact
generated paths owned by ticket 0936.

Remove PubMed's duplicate fixture-oriented projection structures where runtime
`RequestPlan` assertions can express the same contract. Tests must inspect the
production plan rather than retain a parallel representation.

## Done when

- Clean-tree, added-file, renamed-file, one-line growth, lowered baseline,
  unauthorized raise, generated-path, and symlink/path fixtures prove the
  inventory cannot be bypassed.
- Dead-code fixtures cover module/file/item attributes, reason drift, deleted
  exceptions, generated bindings, and a near-match path.
- The largest named modules receive no broad refactor solely to satisfy this
  ticket; future product work performs local extraction when it would otherwise
  exceed a pinned baseline.
- Canonical lint runs both ratchets once and docs explain the narrow exception
  process without offering a blanket refresh command.

## Authorized test changes

Design commits may add the source-size/dead-code inventories and audits, narrow
allowances, remove duplicate PubMed test projections, and restate quality docs
and tests. Product behavior and public schemas must not change.

The src line ceiling may not rise.
