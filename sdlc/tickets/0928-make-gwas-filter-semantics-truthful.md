---
flow: build
priority: 10
deps: ["0915", "0927"]
---
# Make GWAS filter semantics truthful

`--region` is accepted and advertised but has no request path, so a region-only
search returns a successful empty page. Supplying both gene and trait appends
their independent results, although the user guide says combined filters
narrow the result.

## Filter contract

The current GWAS Catalog client does not implement an interval request.
Reject every non-empty `--region` before transport with a typed unsupported
filter error, and remove region examples from help, list output, schemas, and
documentation. A later interval feature requires its own bounded request plan;
do not approximate a region by returning empty data.

Gene and trait are individually valid. When both are supplied, retrieve each
bounded candidate set and return only associations whose normalized rsID is in
both sets. Apply `--p-value` to that intersection. Preserve the best
association per existing deduplication rules and never turn disjoint inputs
into a union.

The numerical work budget is fixed: at most two provider requests, one gene
leg and one trait leg; at most 50 decoded candidates per leg; at most 100 rows
before intersection; at most 50 normalized rsIDs retained per leg; and at most
the validated 1-50 requested rows returned. Each single-filter call uses one
request and the same 50-row ceiling. Ticket 0927's checked window is the only
pagination window; this ticket must not traverse additional provider pages to
fill an intersection.

## Done when

- Region-only and region-plus-other-filter commands fail before the local
  transport sees a request, in both CLI and MCP paths.
- Local gene and trait responses cover overlapping and disjoint rsIDs; combined
  output is exactly the intersection in stable order.
- A p-value boundary is applied after intersection and does not admit a row
  from only one source leg.
- A RequestPlan/transport observation pins the one/two-request paths and every
  numerical candidate, retained-ID, pre-intersection, and output ceiling. No
  unbounded fan-out is introduced.
- JSON, Markdown, help, typed catalog, and GWAS user documentation state the
  same supported filters and AND semantics.
- No routine test reaches GWAS Catalog.

## Authorized test changes

Design commits may restate GWAS Clap/help, discovery catalog, entity search,
local source construction, response, rendering, and documentation tests in
`src/cli/gwas`, `src/entities/variant/gwas.rs`, `src/sources/gwas`, and the
public GWAS pages. Existing single-gene, single-trait, p-value, pagination, and
source-provenance assertions remain covered.

The src line ceiling may rise by at most 180 lines.
