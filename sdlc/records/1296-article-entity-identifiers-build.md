# 1296 build: give article entities identifiers

Ticket: sdlc/tickets/1296-give-article-entities-identifiers.md
Branch: tickets/1296-article-entity-identifiers, sha b57b8fc6e
Built on a branch queue; reviewed fresh.

## Changes

- Annotation rows carry namespace and identifier: NCBIGene for genes,
  MESH/OMIM for diseases and chemicals via the prefixed registry identity,
  rsID preferred then HGVS for variants, never the tmVar composite.
  Placeholders carry nothing. Fields serialize only when present.
- Aggregation keys on (text, identifier), so the same text under
  different identifiers stays separate and identical entities never merge
  across texts.
- Passage positions serialize only behind article entities --full (JSON);
  every default path keeps its old shape.
- Markdown rows print a get command only where BioMCP accepts the
  identifier form; gene and chemical rows keep text search.

## Proof

- Code review ACCEPT; the one P2 (hand-updated zero-coupling digest)
  closed by a local ratchet run, 35 passed.
- Spec page pins JSON rows, compact-versus-full shapes, and markdown
  commands from the recorded PubTator3 capture of PMID 30738221.
- 995 targeted tests; make lint green; spec-pr article lane green.

## Deferred gaps

- MeSH-to-MONDO mapping stays deferred; the crosswalk serves the outcome.
- get gene accepting NCBI Gene identifiers is separate-ticket territory.
