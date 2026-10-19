---
priority: 2
---

# 2274 — Resolve common disease names past the first page

Status: OPEN.

Milestone: 0.9.4

Depends on: 2268

Reviews: revision 7562c27, accept

Reviews: revision 7562c27, accept

Reviews: revision d2b41ca, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/pm.json`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/article/fulltext_view.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/abstracts.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/fulltext.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/disease/resolution.rs`, `src/entities/disease/resolution/paging.rs`, `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/europepmc.rs`, `src/sources/mychem.rs`, `src/sources/pubmed.rs`, `src/transform/article.rs`, `src/transform/article/anchors.rs`, `src/transform/article/federation.rs`, `src/transform/drug.rs`, `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_article_abstract_recovery.py`, `tests/test_article_fulltext_quality.py`, `tests/test_article_metadata_recovery.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

The exact-term disease resolver settles common names such as melanoma even when MyDisease returns more hits than one page holds, so disease-dependent features such as the article disease filter (2267) work for common cancers.

## Evidence

2267's MyDisease capture for "melanoma" returns 90 hits with 50 supplied, and the resolver refuses completeness, so Semantic Scholar rows become unavailable for one of the most common cancer queries.

## Keeps

Keep every ambiguity safeguard (MM, MF, HD and the 2017 refusals). Bound the extra paging or use an exact-term query so a common name costs at most a small fixed number of requests within the deadline.

## Proof

A replay test resolves melanoma from the 90-hit capture; the existing ambiguous-abbreviation refusals still refuse; the request count stays within the stated bound.

## Implementation evidence

Fresh developer completed 7562c27e21ed31211b3ad35d8c326c8a3f505506. Red job 2274/1792438683863-48711 reproduced the incomplete first-page refusal. Green 2274/1792438791281-86968 passed the affected 71 tests, formatting, warnings-denied all-target Clippy and quality checks. Fresh reviewer accepted that exact revision without changes, verified the retained patch, and did not repeat unchanged proof.

Paging uses at most four complete pages of 50 candidates and one detail request within one shared deadline. Refuse excessive or changing totals, short pages, duplicate IDs, incomplete replies and ambiguity. Existing transport retry policy remains; the logical-call bound does not claim a smaller transport-attempt count. Public melanoma replies contain 90 hits across two pages. Other bounded count probes establish counts only, not canonical resolution. Combined article CLI proof and joint landing with 2267 remain required.

Keep: experiments/2088-biomcp-2274-disease-pages — retain actual provider pages, red and green proof, checked patch and checksums.

Landing integration: this branch contains the reviewed 2268 prerequisite so the runtime batch uses one final gate; this records integration order and adds no product outcome.
