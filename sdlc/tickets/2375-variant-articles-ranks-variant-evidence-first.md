# 2375 — Variant articles ranks variant evidence first

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

The first page of `variant articles` leads with papers about the variant: the variant in the title or abstract, and the landmark discovery and clinical papers, ahead of papers that only mention the gene.

- Changes: `src/entities/article/variant_search/rank.rs`, `src/entities/article/ranking.rs`, `tests/test_variant_article_evidence_ranking.py`, `tests/variant_article_evidence_replay.py`, `tests/fixtures/article/evidence-ranking/akt1-e17k.json`, `tests/fixtures/article/evidence-ranking/README.md`, `spec/fixtures/run-variant-article-entity-fixture.sh`. The pool owner integrates the stage calls in its shared orchestration file.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- The 2007 Nature discovery paper PMID 17611497 sits near union rank 388 (PubMed about 90). 5 of union's first 10 are confirmed variant-evidence papers, against 9 of PubMed's first 10.

## Proof

- Outside-in replay test: for AKT1 E17K, at least as many of the first 10 rows are confirmed variant-evidence papers as PubMed's first 10, and 17611497 and 33123537 rank in the first page. Rank on evidence the result already carries; add no hand-picked list.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.
