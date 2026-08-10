---
flow: build
priority: 10
deps: ["0909"]
---
# Honor article source selection during enrichment

Issue owned by this ticket:
`sdlc/issues/semantic-scholar-enrichment-ignores-source-selection.md`

Article planning currently distinguishes candidate sources, but the shared
finalizer can contact Semantic Scholar for rows produced by an explicitly
selected PubMed, PubTator, or Europe PMC route. Output can then say
`semantic_scholar_enabled: false` despite the outbound request.

## Source-control contract

Represent candidate retrieval and row enrichment as separate explicit source
plans. An explicit `--source pubmed`, `pubtator`, `europepmc`, or `litsense2`
permits traffic only to that selected source and any already documented
identity-resolution dependency for that route; it performs no Semantic Scholar
search, batch lookup, or row enrichment. `--source semanticscholar` and the
default/all route may use Semantic Scholar according to their documented plan.

Every response and debug plan reports candidate sources, enrichment sources,
and the actual outcome of every provider BioMCP attempted. A provider absent
from the plan is absent from the request log and is not reported as attempted,
empty, or failed. Do not use a candidate-only Boolean to describe all outbound
traffic.

## Done when

- Local counting servers cover every explicit article source and the default
  route. Excluded Semantic Scholar receives exactly zero requests, including
  when selected rows have PMIDs that could be enriched.
- The default/all route still performs its documented optional enrichment and
  records its real status.
- JSON, Markdown, `--counts-only`, `search all`, and `--debug-plan` agree on
  every attempted source; adding debug output never changes the request plan.
- Query text, article identifiers, credentials, and provider URLs remain
  protected by ticket 0909's logging boundary.
- Article help, source documentation, the Semantic Scholar key guidance, and
  output-footprint fixtures describe the same source behavior.
- `RUN.md` and the canonical API-key guide both state that Semantic Scholar
  TLDR can run anonymously and that `S2_API_KEY` raises quota; neither calls
  the key mandatory.
- No routine test reaches a public provider.

## Authorized test changes

Design commits may restate article planner/finalizer and source-status tests in
`src/entities/article/planner.rs`, `src/entities/article/enrichment.rs`,
`src/entities/article/search.rs`, `src/cli/article/tests`, search-all tests,
`tests/test_output_footprint_benchmark.py`, and the corresponding article,
Semantic Scholar, and operator documentation. Existing result identity,
deduplication, and degraded-provider assertions must remain covered.

Delete the named issue when this ticket lands.

The src line ceiling may rise by at most 200 lines.
