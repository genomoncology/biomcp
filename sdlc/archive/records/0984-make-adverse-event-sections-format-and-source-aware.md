---
base: 158ec070
head: 76e487c1
---

FAERS section requests now validate names before retrieval and produce the
same bounded projection in Markdown, JSON, raw MCP, and typed MCP. Full and
`all` output remain compatible; subsets preserve identity, selected empty
arrays, filtered provenance, shared guidance, and evidence without unrelated
navigation. Device reports reject named sections after source resolution, and
batch references state that adverse-event sections are unsupported.

Typed adverse-event sections accept duplicates idempotently while other
entities retain uniqueness enforcement. Extracted modules reduced the pinned
large-file baselines, canonical lint passed, and independent review accepted
the final process and MCP contracts.
