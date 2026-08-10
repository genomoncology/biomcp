---
flow: build
priority: 10
---
# Coordinate-carrying structs take a new field without editing tests

Ticket 0689 has been refused eight times. Two of those refusals, and every
future one of the same kind, come from this:

    refused: reverting those test changes makes the implementation fail to
    compile with 42 missing-field errors

`Gene` derives `Debug, Clone, Serialize, Deserialize` — not `Default` — and
its tests build it with seventeen exhaustive `Gene { … }` literals and zero
`..Default::default()`. In Rust an exhaustive struct literal must name every
field, so **adding one field breaks every one of those literals at compile
time.**

The build flow reserves test edits for `design:` and `design-review:`
commits. So a `code:` commit that adds a field cannot compile, and cannot
legally make itself compile. There is no move that satisfies both rules. That
is not an agent mistake; it is the codebase and the flow disagreeing.

This ticket removes the disagreement. It changes no behavior.

## Done when

- Every struct listed below derives or implements `Default`.
- No test constructs any of them with an exhaustive struct literal; each uses
  `..Default::default()` for the fields it does not care about.
- A ratchet in the test suite fails if an exhaustive literal of these types
  reappears in test code, so this cannot silently rot back.
- Behavior is identical: `make lint`, `make test`, and `make spec` pass, and
  no fixture, schema, or captured output changes. If any output changes, the
  change is wrong.

## The structs

- `Gene` and `GeneSearchResult` (`src/entities/gene.rs`)
- `VariantNormalizationServiceResult` (`src/entities/variant/normalization.rs`)
- The coordinate-carrying variant structures in
  `src/entities/variant/mod.rs`, `get.rs`, and `structure.rs`

If a struct in that list genuinely cannot have a sensible `Default` — a field
with no meaningful empty value — say so in the design and leave it out with
the reason. Do not invent a misleading default to satisfy the checklist.

## Tests you are authorized to restate

This ticket is a test refactor, so the design stage rewrites these files and
that is expected, not a traceability violation:

    src/cli/tests/outcome.rs
    src/cli/variant/tests.rs
    src/entities/variant/get/tests.rs
    src/entities/variant/search/tests.rs
    src/render/markdown/evidence/tests.rs
    src/render/markdown/gene/tests/extended.rs
    src/render/markdown/gene/tests/rendering.rs
    src/render/markdown/related/tests/gene_drug.rs
    src/render/markdown/root_tests.rs
    src/render/markdown/sections/tests.rs
    src/render/markdown/variant/tests.rs
    src/sources/disgenet/tests/mod.rs

Restate construction only. Do not weaken or delete an assertion: if a test
checks a value today, it checks the same value afterwards.

## Why this runs before 0689

0689, 0899 and 0900 all add a field to these structs. Each of them hits this
wall independently. Landing this first means those three become ordinary
tickets instead of impossible ones.

The src line ceiling may rise by at most 40 lines — the `Default`
implementations and the ratchet.
