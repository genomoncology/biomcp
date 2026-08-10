---
flow: build
priority: 8
deps: ["0915"]
---
# Make protein offset pagination truthful

`search protein BRAF --limit 1 --json` reports a provider total of 151 but
`has_more: false`, even though `--offset 1` returns another row. Human output
offers that next offset, so the two renderers disagree. The JSON path currently
requires a provider cursor token even when the public continuation is an
offset.

## Pagination contract

For an offset request, `has_more` is true whenever
`offset + returned < total`, whether or not the current provider page token can
be exposed. `next_page_token` remains null when BioMCP cannot safely expose a
cursor. JSON also carries `next_offset` when more rows are reachable; it is
`offset + returned` and is null at the end.

`next_offset` is a protein response field, not a new serialized field on the
shared `PaginationMeta` used by other entities. Use a protein-specific
pagination projection, or an equivalent omitted-by-construction wrapper, and
prove the exact JSON schemas of article, drug, disease, gene, trial, pathway,
PGx, and every other existing pagination consumer remain unchanged.

For an unknown total, BioMCP fetches one extra row or retains an internal
provider continuation to decide `has_more` without exposing an invalid token.
It returns at most the requested limit. JSON and Markdown use the same
pagination value.

## Done when

Local UniProt pages cover the first page, a middle offset, an exact final page,
an empty page, an unknown total, and a mid-provider-page stop. Following every
advertised `next_offset` reaches the next distinct row without a gap or
duplicate. Schema snapshots prove non-protein pagination did not gain a null or
populated `next_offset`. No routine test calls UniProt.

## Authorized test changes

Design commits may restate protein pagination expectations in
`src/cli/protein/tests.rs`, `src/entities/protein.rs`,
`src/cli/shared.rs`, response-schema snapshots, and protein JSON/Markdown
renderer tests. The shared
`cursor_without_token_never_promises_a_next_page` test may be narrowed or
restated for true cursor-only callers; other cursor contracts remain intact.

The src line ceiling may rise by at most 120 lines.
