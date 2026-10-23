# 2373 — Variant articles includes every plain PubMed hit

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

For a gene plus protein change, `variant articles` returns every PMID that a plain PubMed search for the same gene and change words returns. Plain PubMed is the floor; BioMCP adds to it and never drops from it.

## Changes

- Changes: `src/entities/article/variant_search.rs`, `src/entities/article/variant_search/sources.rs`, `src/entities/article/backends.rs`, `src/entities/article/query.rs`, `src/sources/mod.rs`, `src/sources/pubmed.rs`, `src/sources/pubmed/tests/construction.rs`, `src/sources/pubmed/tests/live.rs`, `src/sources/pubmed/tests/parsing.rs`, `tests/test_variant_article_pubmed_floor.py`, `tests/fixtures/article/pubmed-floor/**`, `tests/feature-links.json`.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- PMID 33123537, PubMed rank 2, 'Effect of AKT1 (p. E17K) Hotspot Mutation on Malignant Tumorigenesis and Prognosis', is absent from all 35 union pages and from the first 50 rows of --strategy lexical, --strategy annotation and `search article -k 'AKT1 E17K'`. 20 of PubMed's 192 PMIDs never appear.

## Proof

- Outside-in replay test over the AKT1 E17K PubMed ESearch fixture: every PMID in it appears in the paged variant articles result. A contract test states the superset rule for the default strategy. Trace why each of the 20 missing PMIDs dropped, and fix each cause, not each paper.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.
