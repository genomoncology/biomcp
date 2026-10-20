---
flow: build
priority: 7
---

# Rank an exact gene symbol before alias matches

## Outcome

Eligible gene-symbol searches now acquire one bounded complete MyGene set when
the provider total is at most 50, apply the existing filters, promote exact
canonical symbols ahead of alias-only matches, deduplicate first-wins, and then
page the stable result. Overflow and ineligible free-text searches retain their
previous bounded request behavior. CLI, raw MCP, and typed MCP therefore build
their first detail command from the same first result.

## Review and evidence

Independent code review rejected four successive proof gaps before accepting
the final revision. The repaired executable contract covers totals 50 and 51,
all paging boundaries, exact request counts, filtering and Lucene adversaries,
missing and duplicate identities, stable no-exact ordering, hostile provider
symbols through a real shell, complete CLI/MCP Markdown and JSON, and unchanged
generated help, schemas, and tool inventory. The shared next-command serializer
is unchanged.

The final reviewed tree passed `make lint`, `make test`, and `make spec`.
`make test` reported 948 Python contracts passed with 3 skipped and completed
the offline Rust archive and strict documentation lanes. The source package
contains exactly 1,300 paths, `src/entities/gene.rs` remains exactly 3,859
lines, the external-project coupling check passed, and `git diff --check`
passed.

## Boundary

The change is confined to bounded ordering and deduplication in the gene entity
plus the existing gene specification and provider fixture. It does not change
gene-get identity, query construction, filters, CLI or MCP schemas, rendering,
next-command construction, dependencies, or other entities. No live-provider
verification was used.
