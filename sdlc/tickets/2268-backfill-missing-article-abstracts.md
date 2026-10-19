---
priority: 2
---

# 2268 — Backfill missing article abstracts

Status: OPEN.

Milestone: 0.9.4

Depends on: 2269

Reviews: revision f33558c, accept

Reviews: revision 8224ddd, accept

Reviews: revision 85c013a, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/pm.json`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/article/fulltext_view.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/abstracts.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/fulltext.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/europepmc.rs`, `src/sources/mychem.rs`, `src/sources/pubmed.rs`, `src/transform/article.rs`, `src/transform/article/anchors.rs`, `src/transform/article/federation.rs`, `src/transform/drug.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_article_abstract_recovery.py`, `tests/test_article_fulltext_quality.py`, `tests/test_article_metadata_recovery.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

Article search rows under `--full` and `get article` carry the abstract whenever any queried source holds it.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R7: `biomcp --json search article -g CXCR2 -k antagonists --type review --full --limit 5`: three rows found only by Europe PMC have no abstract_snippet, though Europe PMC resultType=core returns abstractText (998 and 1,602 characters for 36780833 and 42423773). R8: `search article -d glioma -k '"triple-negative gliomas"' --source pubmed --full` returns PMID 22890969 with no snippet; PubMed EFetch holds a 1,654-character abstract. R9: `biomcp --json get article 41790359` has no abstract_text; PubTator3 biocjson has a zero-length abstract passage, while PubMed EFetch and Europe PMC core hold it.

## Keeps

Treat an empty abstract as missing and fall back. Stay within the request deadline; batch backfill requests.

## Proof

Replay tests for all three cases show the abstract present with its source.

## Implementation evidence

Fresh developer completed f33558cfd4dba88debc3d522c48bd75c7a17f3ff. Current main failed all four public cases; accepted 2273 failed three, while get 41790359 already passed through 2273. Final M5 job 2268/1792439264613-19661 passed nine CLI controls, 64 affected Rust tests, formatting, Ruff, warnings-denied Clippy and quality checks. Fresh reviewer accepted that exact revision and independently matched every changed source and test with the checked files; no redundant tests ran.

Europe PMC requests core results and retains nested journal metadata. Visible PubMed rows recover missing abstracts in bounded batches through the shared citation parser and existing search deadline. Cleanup preserves literal comparisons and decimal characters. The visible_metadata, backends and search modules remain unchanged. Private direct-provider replies confirmed the missing-lite/core abstract and journal shapes; only synthetic regressions are public. No full landing gate is claimed.

Keep: experiments/2090-biomcp-2268-abstract-recovery — retain actual replies, research outputs, patches, hashes and red/green job logs.

Landing integration: this branch contains the reviewed 2269 prerequisite, so the accepted runtime batch can use one final gate. The dependency records that integration order; it adds no product outcome.
