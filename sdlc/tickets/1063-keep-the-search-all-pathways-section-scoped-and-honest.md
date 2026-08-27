---
flow: build
priority: 10
---

# Keep the search-all Pathways section scoped and honest about partial failure

Filed from `sdlc/issues/2026-08-26-search-all-pathways-ignores-query.md`,
which carries the full verified mechanism — read it first.

In brief: `search all --gene BRAF --disease melanoma` can render a Pathways
section of WikiPathways rows unrelated to the query (AML, Alzheimer's).
Verified cause: the section passes the gene symbol as a free-text query;
`finalize_pathway_search_results` (`src/entities/pathway.rs`) swallows
per-source errors whenever any source answers; and `push_ranked_hits`
retains zero-relevance (tier-0) rows. When Reactome and KEGG time out and
WikiPathways answers, WP's gene-membership results fill the section in
upstream order, reading as an alphabetical list with no relationship to the
query. Every other section of the same card is correctly scoped.

## Done when

- The Pathways section never presents rows that have no textual relationship
  to the query — zero-relevance rows are dropped or the design settles and
  documents an explicit relevance floor; silently filling the section with
  one source's noise is what this ticket removes.
- Partial source failure is surfaced the way the card already handles it
  elsewhere: the GWAS section prints its timeout, keeps the card alive, and
  offers the direct retry command. Pathways must do the same when sources
  are lost, instead of rendering WP-only results as if they were the answer.
- The fix holds for the gene-anchor section of `search all` and the plain
  `search pathway` surface consistently — one relevance policy, not two.

Filed as build, not quickfix: the suite is green; the fault is absent
assertions plus an aggregation policy, and the proof must be authored.
