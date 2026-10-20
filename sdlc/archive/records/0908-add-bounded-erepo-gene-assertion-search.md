---
base: 4693d157
head: c6af99b7
---

`variant erepo --gene` now searches the production classifications endpoint
with mutually exclusive CAID/gene inputs, a default limit of 25, a maximum of
100, and a non-negative offset. It requests one extra row and returns truthful
`has_more`, `returned`, paging fields, and `total: null` rather than inventing
a provider total.

Compact results retain CAID, gene, condition, classification, guideline,
expert panel, publication date, bounded HGVS previews with the full count, met
evidence codes, and typed truncation markers. The typed MCP tool uses the same
execution and response boundary. A dated, byte-faithful PTEN capture and local
HTTP fixture prove request shape, second-page access, rendering, and CLI/MCP
parity without routine public-network access.

The complete lint, routine test, executable specification, and all-feature
gates passed as part of the three-ticket ClinGen batch.
