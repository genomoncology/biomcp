---
flow: build
priority: 10
---

# Add the author as a searchable, gettable entity with paper pivots

## Use case

BioMCP's article family stops at the paper. A working researcher — and
the marketing/outreach work that surrounds BioMCP — repeatedly needs the
next three questions, and today BioMCP cannot answer any of them:

- Who wrote this paper, where do they work now, and how do I reach them?
- What else has this person written?
- Who is the established expert on this topic?

Two concrete drivers:

1. Outreach research (workspace `experiments/020-arxiv-endorser-hunt` and
   the `repos/rolodex` CLI) needed author search, affiliation, metrics,
   and author-to-paper traversal so badly that the tool calls the
   Semantic Scholar Graph API directly with `curl`. Every one of those
   calls is a capability BioMCP should own.
2. The v0.8.16 evaluation (BioMCP paper, pass 5) scored literature
   navigation the weakest workflow, partly because there is no way to
   move from a paper to its people.

## What done looks like

Following the existing grammar — the same shape as every other entity
family, not a new pattern:

- `biomcp search author "Renato Umeton"` returns a compact list of
  matching authors with identifiers and current affiliation.
- `biomcp get author <id>` returns a compact author card; JSON output
  carries `_meta.next_commands` and evidence URLs back to the upstream
  record, like every other `get` surface.
- `biomcp author papers <id>` pivots to their works.
- `biomcp article authors <pmid-or-arxiv-id>` (or the equivalent pivot
  helper naming the design prefers) moves from a paper to its author
  records, closing the paper-to-person gap the evaluation flagged.
- Sections expand deeper facets the way other families do (metrics,
  affiliations) without changing the compact default card.

## Upstream and hard choices, settled

- **Primary upstream: the Semantic Scholar Graph API.** BioMCP already
  holds an optional `S2_API_KEY` for citation-graph features and already
  labels key-gated helpers explicitly; the author family follows that
  same pattern. The endpoints also work keyless at shared-pool limits,
  so the no-auth surface degrades instead of disappearing.
- **OpenAlex is a complement, not this ticket.** If the design wants
  OpenAlex for ORCID and citation enrichment, that lands as its own
  follow-up ticket so this one stays one behavior.
- Author identity is the upstream's id (S2 authorId), the same way
  variants carry ClinVar ids and trials carry NCT ids. No local
  identity resolution in this ticket.

## Example calls this feature wraps

These are live today with an optional key and are the contract the
design should wrap (`$S2_API_KEY` is the existing env var):

Author search — names, current affiliation, counts:

```bash
curl -G 'https://api.semanticscholar.org/graph/v1/author/search' \
  --data-urlencode 'query=Renato Umeton' \
  --data-urlencode 'fields=name,affiliations,homepage,paperCount,citationCount,hIndex,externalIds' \
  -H "x-api-key: $S2_API_KEY"
```

An author's papers — the pivot:

```bash
curl 'https://api.semanticscholar.org/graph/v1/author/50978539/papers?fields=title,year,venue,externalIds,abstract,openAccessPdf,authors&limit=50' \
  -H "x-api-key: $S2_API_KEY"
```

Paper-to-authors — the reverse pivot the evaluation missed (works from
an arXiv id; PMID and DOI forms exist too):

```bash
curl 'https://api.semanticscholar.org/graph/v1/paper/arXiv:2110.01406?fields=title,year,authors.name,authors.affiliations,authors.paperCount,authors.hIndex' \
  -H "x-api-key: $S2_API_KEY"
```

The OpenAlex complement, for the follow-up ticket (no signup; the
mailto is the whole auth):

```bash
curl 'https://api.openalex.org/authors?search=Renato%20Umeton&mailto=ian@imaurer.com'
```

## Notes

- Compact cards matter here more than usual: author lists run long, and
  the v0.8.16 token measurements showed compact output is the property
  agents actually consume.
- Filed as a draft because the command naming (`author papers` vs
  `get author papers`) and the section split are design-stage choices;
  promote when the shape is agreed.
