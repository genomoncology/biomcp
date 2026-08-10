---
flow: build
priority: 8
deps: ["0915"]
---
# Make PGx section requests focused and bounded

`get pgx CYP2D6 recommendations` currently retrieves and renders up to one
hundred base interaction pairs before adding as many as fifty recommendations.
The docs describe recommendations as a focused section, but the command still
does unrelated API and output work.

## Command contract

Add `interactions` to the named PGx sections and support `--limit`, `--offset`,
and `--full` on `get pgx`:

- default no-section output requests and returns at most 10 interaction rows;
- a named section returns identity/provenance plus only that section unless
  more sections are explicitly named;
- `--limit` defaults to 10 and is limited to 1-50 per requested list section;
- `--offset` applies to every requested list section and defaults to zero;
- each section has its own `returned`, `total` when exact, `has_more`, and
  `next_offset` metadata; and
- `--full` selects all sections and raises their per-section limit to 50. It
  does not remove the hard cap; further rows use continuation.

For a likely gene or drug, call the requested section endpoint directly. Use at
most one limit-one pair lookup only when identity or gene-versus-drug routing
cannot otherwise be established. Do not fetch or build a full interaction list
merely to route recommendations, frequencies, guidelines, or annotations.
Provider requests fetch limit plus one when no exact total is available and
render no more than limit.

## Done when

- Clap accepts flags in documented positions and rejects zero/oversized limits
  before transport.
- Local request observations prove a recommendations-only gene call performs
  no 100-row pair request and emits no interactions.
- Default, one-section, multi-section, offset, and full cases stay within the
  stated request and output budgets.
- Recorded CPIC and PharmGKB bytes pass through production decoders.
- JSON and Markdown expose the same per-section continuation and source state.

## Authorized test changes

Design commits may restate PGx argument/section behavior in
`src/cli/pgx/tests.rs`, `src/entities/pgx.rs`, `src/sources/cpic.rs`, PGx
render tests/templates, `spec/entity/pgx.md`, and corresponding list/source
documentation contracts. Existing search-PGx filters and typed provider
outcomes remain covered.

The src line ceiling may rise by at most 300 lines.
