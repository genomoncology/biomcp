---
flow: build
priority: 10
deps: ["0951"]
---
# Make GWAS pagination fail safely

`search gwas --limit 1 --offset 200` reaches a `clamp` call whose lower bound
is above its upper bound and aborts the process. Similar arithmetic is used by
gene, trait, and study fallback requests.

## Pagination contract

Until GWAS Catalog cursor traversal is designed, BioMCP supports only windows
where checked `offset + limit` is at most 50. Validate that bound, integer
overflow, and the existing 1-50 limit before constructing a client or sending
a request. Every other window returns a normal typed invalid-argument error;
no user input may panic the CLI or MCP server.

Within the supported window, request at most one extra row when provider
capacity permits and expose only followable continuation. At the 50-row work
boundary, do not advertise an unusable next offset. Instead report
`truncated_by_provider_budget: true`, `has_more: false`, and a null
`next_offset`, with human guidance to narrow the filters. This does not claim
the biomedical result set is exhausted.

These fields are a GWAS-only serialization. GWAS JSON contains exactly:

```json
{
"_meta": {
  "pagination": {
    "limit": 10,
    "offset": 40,
    "returned": 10,
    "has_more": false,
    "next_offset": null,
    "truncated_by_provider_budget": true
  }
}
}
```

At true exhaustion, `truncated_by_provider_budget` and `has_more` are false and
`next_offset` is null. For a followable page, truncation is false, `has_more`
is true, and `next_offset` is the checked `offset + returned`. No other
combination is valid. Human output uses exactly: `More GWAS rows may exist, but
BioMCP's 50-row provider budget was reached. Narrow the filters; no next offset
is available.` Existing JSON and Markdown pagination for every non-GWAS entity
remain byte-for-byte unchanged.

## Done when

- Process-level CLI and MCP cases cover offsets 0, 49, 50, 200,
  `usize::MAX`, exact window 50, and window 51 for gene and trait filters.
- Every invalid case exits normally with the stable error contract before
  transport; none aborts or unwinds across the process boundary.
- Valid first and middle pages contain distinct rows with no gap or duplicate.
- JSON and Markdown distinguish provider-budget truncation from exhaustion.
- Schema snapshots prove no non-GWAS response gained any new or null field.
- Request construction uses checked arithmetic and never calls `clamp` with
  runtime-derived minimum and fixed maximum in the wrong order.
- No routine test reaches GWAS Catalog.

## Authorized test changes

Design and code commits may add the failing process and local-source cases and restate
GWAS pagination expectations in `src/entities/variant/gwas.rs`,
`src/cli/gwas/tests.rs`, GWAS render tests, and GWAS documentation. Existing
p-value, result mapping, and source-attribution assertions remain covered.

The src line ceiling may rise by at most 100 lines.
