---
flow: build
priority: 10
---

# Connect authors and papers with both pivots

## Where the author family stands today

`search author -q <name> --source semanticscholar` and
`get author semanticscholar:<id>` exist — provider-exact Semantic Scholar
records, landed 2026-07-15 (`db3f8ff4`). What the family cannot do is move:
the CLI reference lists publication, coauthor, topic, and
affiliation-filter operations as future work, and `get author`'s JSON
already carries `evidence_urls` and source status but emits `next_commands`
as an empty vector (`src/entities/author/detail.rs`) — there is nothing for
the card to point at. The paper's own gap analysis (`notes/biomcp/
biomcp-paper/plan.md`) flags weak next-command/HATEOAS guidance as a known
BioMCP weakness; the author card is its starkest instance.

The missing moves, in researcher terms:

- From a person to their body of work — "what else has this person
  written?" — impossible today.
- From a paper to its people — "who wrote this, where do they work, how do
  I reach them?" — impossible today; the article family stops at the
  paper.

The outreach history is the grounding: the endorser hunt
(`experiments/020-arxiv-endorser-hunt`) and the `repos/rolodex` CLI called
the Semantic Scholar Graph API directly with `curl` for exactly these two
traversals. Every one of those calls is a capability BioMCP should own.

## What done looks like

Follow the existing pivot grammar — the same shape as the article family's
`article citations <id>` / `article references <id>` / `article
recommendations <id>` — not a new pattern:

- `biomcp author papers <id>` pivots from an author record to their works,
  compact by default, with `_meta.next_commands`, evidence URLs, and
  pagination, like every other list surface.
- `biomcp article authors <id>` pivots from a paper (any id form the
  article family already accepts) to its author records — the reverse
  pivot, closing the paper-to-person gap.
- `get author <id>` gains truthful `next_commands` pointing at the new
  pivot, so the card finally leads somewhere.
- Command naming (`author papers` vs `get author papers`) and any section
  split are design-stage decisions; the grammar precedent above is the
  default, not a ruling.

## Boundary for the design stage — do not rebuild what exists

Search and detail already exist and stay as they are, including their
sanitization (the author detail path already strips private fields —
email, homepage-adjacent demographics, ORCID — behind provider-exact
records; the pivots must not leak what detail sanitizes). IDs remain
provider-qualified upstream ids (`semanticscholar:<authorId>`), the same
convention as today; no local identity resolution, no same-name merging.

## Upstream and hard choices, settled

- **Primary upstream: the Semantic Scholar Graph API.** BioMCP already
  holds an optional `S2_API_KEY` for the citation-graph pivots and labels
  key-gated surfaces explicitly; the author pivots follow that same
  pattern, including degrading to the shared pool keyless rather than
  disappearing.
- **OpenAlex is a complement, not this ticket.** If the design wants
  OpenAlex for ORCID and citation enrichment, that lands as its own
  follow-up ticket so this one stays one behavior.

## Example calls this feature wraps

These are live today with an optional key and are the contract the design
should wrap (`$S2_API_KEY` is the existing env var):

An author's papers — the forward pivot:

```bash
curl 'https://api.semanticscholar.org/graph/v1/author/50978539/papers?fields=title,year,venue,externalIds,abstract,openAccessPdf,authors&limit=50' -H "x-api-key: $S2_API_KEY"
```

Paper-to-authors — the reverse pivot (works from an arXiv id; PMID and DOI
forms exist too):

```bash
curl 'https://api.semanticscholar.org/graph/v1/paper/arXiv:2110.01406?fields=title,year,authors.name,authors.affiliations,authors.paperCount,authors.hIndex' -H "x-api-key: $S2_API_KEY"
```

The OpenAlex complement, for the follow-up ticket (no signup; the mailto
is the whole auth):

```bash
curl 'https://api.openalex.org/authors?search=Renato%20Umeton&mailto=ian@imaurer.com'
```

## Out of scope

- Topic-to-expert discovery ("who is the established expert on X") — that
  is topic search, a different upstream capability already listed as
  future work in the CLI reference.
- ORCID resolution or cross-provider identity merging.
- OpenAlex integration (follow-up ticket).

## Notes

- Compact cards matter here more than usual: author paper lists run long,
  and the v0.8.16 token measurements showed compact output is the property
  agents actually consume.
- Filed as a draft pending Ian's shape discussion (including Renato's take
  on the M3 angle); promotion is approval.
