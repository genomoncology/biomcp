---
flow: build
priority: 4
deps: ["0915"]
---
# Add bounded ERepo gene assertion search

This is the gene-sweep slice removed from ticket 0881. It is deliberately
bounded; returning every assertion in one response is not acceptable.

## Command contract

Add:

    biomcp variant erepo --gene PTEN --limit 25 --offset 0 --json

CAID detail/batch input and --gene are mutually exclusive. The default limit
is 25, the maximum is 100, and offset must be non-negative. The production
request uses the ERepo classifications endpoint with gene, matchLimit, and
matchSkip. Request one extra row, return at most limit rows, and derive
has_more from the extra row. ERepo does not provide an exact total, so JSON
must report total as null rather than turning returned rows into a false
total.

Each compact result carries CAID, gene, condition, classification outcome,
guideline label, expert-panel identity, publication date, a bounded HGVS
preview with a full count, and met evidence-code names. Human output uses the
same bounded page. Full assertion detail remains available through the
existing CAID command.

Criterion-code filtering and downloading every result are out of scope. They
need their own paging/work-budget design if later justified.

## Done when

- Clap rejects mixed CAID/gene input, zero or oversized limits, and negative
  offsets before transport.
- A RequestPlan test pins gene, matchLimit=limit+1, and matchSkip=offset.
- A dated receipted PTEN response passes through the production decoder.
- A local HTTP test proves the executor sends the plan and a second page is
  reachable.
- JSON and Markdown prove compact fields, total null, returned, offset,
  limit, and truthful has_more.
- Empty and provider-failure outcomes are typed and source-attributed.
- No routine test reaches ERepo.

The design stage may restate existing ERepo CLI/parser fixtures required by
the mutually exclusive input grammar. Mechanical construction fixes may land
with implementation while preserving behavioral assertions.

The src line ceiling may rise by at most 240 lines.
