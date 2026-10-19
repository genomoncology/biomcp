---
priority: 2
---

# 2269 — Report article metadata and section completeness truthfully

Status: OPEN.

Milestone: 0.9.4

Depends on: 2273

Reviews: revision 7aec03f, accept

Reviews: revision 3f97f87, accept

Reviews: revision 2d45bf5, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/pm.json`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/article/fulltext_view.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/fulltext.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/mychem.rs`, `src/sources/pubmed.rs`, `src/transform/article.rs`, `src/transform/drug.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_article_fulltext_quality.py`, `tests/test_article_metadata_recovery.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

Article search reports an internal metadata failure in its JSON source status and marks affected rows incomplete, and full-text quality flags agree with the parsed output.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R3: `biomcp search article -d glioblastoma -k "recurrence cell states mesenchymal shift" --full --json --limit 5` warns on stderr that the PubTator3 visible-article metadata fallback failed (internal), yet JSON says complete:true and source_status omits PubTator3; PubTator3 biocjson answers 200 for those PMIDs, so check whether the failure is ours. R4: `biomcp get article 28736637 fulltext --json` (PMC HTML fallback) gives full_text_manifest.quality.has_sections:false while full_text_summary.section_count is 31.

## Keeps

Build on 2235's recovery. Compute flags from the parsed output.

## Proof

Replay tests: the failing fallback appears in source_status with complete:false; the 28736637 case reports has_sections:true. If the PubTator3 failure is BioMCP's request, fix it.

## Implementation evidence

Fresh developer stopped clean at 1ab17167fdb31ac0819f7ecc16e701cb304e8ee9. Quality red job 2269/1792437843392-43138 reproduced has_sections false with a saved heading while the no-heading control passed. Metadata red 2269/1792437928440-12593 preserved recovery but incorrectly reported complete:true. Final job 2269/1792438126231-58882 passed six CLI controls, formatting, Ruff, warnings-denied all-target Clippy, quality boundaries and three affected Rust regressions. Public completeness stays at page level with source-specific failure provenance; no per-row schema field was introduced. Fulltext manifest, summary and outline share heading recognition.

One bounded exact original query, job 2269/1792438202232-80759, returned five results with complete:false and PubTator HTTP400 provenance. The original internal-class failure did not recur, and its original wire request is unavailable, so no deeper historical request cause is claimed. The retained HTTP400 fixture is not presented as the cause of that different internal error. Real recovery data and existing warning behavior remain intact. No full landing gate is claimed. Carried records are approved PM ancestry.

Keep: experiments/2086-biomcp-2269-article-completeness — retain controlled red and green results, original-query output privately, scripts, tested patches and source hashes.

Landing integration: this branch contains the reviewed 2273 prerequisite, so the accepted runtime batch can use one final gate. The dependency records that integration order; it adds no product outcome.
