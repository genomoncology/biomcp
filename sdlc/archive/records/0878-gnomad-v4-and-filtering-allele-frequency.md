---
base: c93361c0
head: d843c47e
---

Variant population retrieval now queries gnomAD GraphQL directly with the
`gnomad_r4` dataset and a resolved GRCh38 coordinate. The result keeps exome
and genome counts separate, derives raw frequencies from their AC/AN values,
preserves ancestry counts and provider flag names, and reports grpmax FAF95
with the bottlenecked-population caveat. The source request and response body
are bounded.

Missing coordinates, absent variants, and provider failures have separate
machine-readable outcomes. A GRCh37-only or unknown-build identity does not
reach gnomAD. JSON, Markdown, provenance, evidence links, schemas, examples,
source documentation, migration notes, and the source-state registry now
describe the same direct v4 contract. The real minimized fixture is covered by
a capture receipt and includes discordant exome/genome filters plus FAF95.

`make lint`, `make test` (2,864 Rust tests and 509 Python contracts), `make
spec`, and `make full-feature-check` passed. The implementation added exactly
360 net `src` lines, matching the ticket ceiling.
