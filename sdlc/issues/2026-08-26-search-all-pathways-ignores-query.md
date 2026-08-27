# search all returns pathways unrelated to the query

`biomcp search all --gene BRAF --disease melanoma` returns a Pathways section of
WP5293 (Acute myeloid leukemia), WP5124 (Alzheimer's disease) and WP2059
(Alzheimer's disease and miRNA effects). None relate to BRAF or melanoma, and
the order looks alphabetical rather than relevance-ranked — as if the pathway
family is listing rather than searching. Every other section in the same card
(Variants, Diseases, Drugs, Trials, Articles) is correctly scoped to the query.

Found while reviewing marketing capture
`repos/mktg/biomcp/drafts/10-ten-cards-one-command/captures/10-search-all.txt`
(BioMCP 0.9.0-dev.6, captured 2026-08-26). The slide does not show the pathway
rows, so nothing is blocked on this, but the section is misleading to a user and
would be embarrassing in a demo.

Verified in code on 2026-08-26 (analysis complete, ticket not yet filed):

- The Pathways section passes the gene symbol as a free-text query
  (`src/cli/search_all/dispatch.rs`, `SectionKind::Pathway` —
  `input.gene_anchor()` becomes `PathwaySearchFilters::query`).
- `search_with_filters` fans out to Reactome, KEGG, and WikiPathways.
  `finalize_pathway_search_results` (`src/entities/pathway.rs`) swallows
  per-source errors whenever ANY source returns hits — so when Reactome
  and KEGG time out and WikiPathways answers, the section renders WP-only
  rows with no signal that two sources were lost.
- WikiPathways search is gene-membership search: it returns pathways whose
  gene lists contain BRAF (AML and Alzheimer's WP pathways include MAPK
  genes), not pathways about the query. All such rows score tier 0 in
  `pathway_title_match_tier` (title does not contain the query), but
  `push_ranked_hits` retains tier-0 rows anyway; with no higher-tier rows
  present they surface in upstream-index order — which reads alphabetical.
- Transience observed: a re-run of the same command had the Pathways
  section time out entirely ("pathway search timed out after 12s") while a
  direct `search pathway --query BRAF` returns six correct Reactome rows
  (R-HSA-167224 "BRAF", mutant-specific variants, hyperphosphorylated
  BRAF). The wrong rows appear exactly when the fast sources are lost.

Classification: code behavior interacting with service flakiness. The data
each source returns is correct; BioMCP's aggregation both hides partial
failure and retains zero-relevance rows.
