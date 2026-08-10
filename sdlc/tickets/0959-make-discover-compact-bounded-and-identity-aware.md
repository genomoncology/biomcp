---
flow: build
priority: 7
deps: ["0957"]
---
# Make discover compact, bounded, and identity-aware

`discover` currently expands weak ontology matches before ranking and can emit
dozens of synonyms and cross-references for each concept. It also ranks an
ambiguous ERBB1 ontology result ahead of the strong gene identity EGFR. Make
the default useful as an agent-selection step rather than a data dump.

## Command contract

Add `--limit` with a default of 5 and range of 1–25, a zero-based `--offset`
with default 0, and an explicit `--full`. Ranking is stable before offset is
applied. Candidate-level validation happens before ranking and paging, so the
offset indexes only the stable validated sequence; malformed candidates are
reported in degradation counts but never consume an offset. Every response
reports `offset`, `limit`, `returned`, `has_more`, and
`next_offset`; its continuation preserves compact/full mode and advances by
exactly the number of complete candidates returned. Overflow fails before
provider work.

The compact default returns, per candidate, stable identity, label, entity
type, source, score/rank reason, at most three synonyms, and at most five
cross-references. Each preview carries exact observed `returned`, `total`, and
`has_more`. The entire default JSON document is capped at 32 KiB; stop before a
complete candidate would cross it and report `budget_truncated` and a
continuation command. Markdown uses the same rows and continuation facts.

`--full` is intentional expansion, not unlimited output. It retains the
1–25 result limit, returns at most 50 synonyms and 100 cross-references per
candidate, and caps the JSON document at 256 KiB with the same whole-record
truncation metadata. No string is split at a byte boundary.

Provider identifiers over 512 UTF-8 bytes are malformed and the candidate is
omitted with a bounded degradation count. Labels are shortened at 512 bytes on
a character boundary and carry `label_truncated`. Compact synonym/xref values
over 256 bytes and full values over 512 bytes are omitted rather than changed;
each preview reports `omitted_oversized`. These field caps guarantee that at
least one well-formed requested candidate can fit an otherwise empty document
budget, so continuation cannot stall on the same row.

Before UMLS atom expansion or other broad enrichment, run the existing strict
typed identity resolvers for recognized gene, drug, disease, and variant names
or identifiers. A strong resolved identity is emitted first with its canonical
identifier and the alias relationship; it is not silently merged with a weak
ontology concept. Rank the remaining candidates before fetching expensive
atoms, and expand only candidates that can fit the requested result window.
ERBB1 must resolve/recommend EGFR ahead of unrelated ontology matches.

## Done when

- Local fixtures cover a typed alias, ambiguous weak matches, hundreds of
  synonyms/cross-references, one oversized identifier, label, synonym, and
  cross-reference, multibyte strings at every byte limit, zero/oversized limits,
  offset overflow, malformed candidates first and between valid rows,
  compact/full rendering, and stable nonrepeating continuation.
- Request recordings prove ranking precedes atom expansion, discarded rows are
  not expanded, and no public provider is contacted by routine tests.
- JSON and Markdown agree on identities, result order, preview counts, budget
  truncation, and the exact command that continues.
- Docs and MCP schemas describe the bounds and never imply `--full` is
  unlimited or that a weak concept is a canonical biomedical identity.

## Authorized test changes

Design commits may restate discover arguments, typed results, ranking and
expansion planning, bounded renderers, local provider fixtures, MCP schemas,
and discover documentation. Do not build a new ontology or generic resolver.

The src line ceiling may rise by at most 220 lines.
