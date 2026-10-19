---
priority: 2
---

# 2270 — Fix drug approvals and regulatory rollups

Status: OPEN.

Milestone: 0.9.4

Depends on: 2267

Reviews: revision 0a61f68, accept

Reviews: revision 93f4d90, accept

Reviews: revision d1edc39, accept

## Outcome

Changes: `CHANGELOG.md`, `benchmarks/output-footprint/run.py`, `docs/user-guide/cli-reference.md`, `sdlc/decisions/2026-10-19-biologic-approvals-match-the-exact-product.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2267-enforce-or-document-the-article-disease-filter.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `sdlc/tickets/2274-resolve-common-disease-names-past-the-first-page.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/article/dispatch.rs`, `src/cli/article/fulltext_view.rs`, `src/cli/article/tests/filters.rs`, `src/cli/discover.rs`, `src/cli/gene/related.rs`, `src/cli/search_all/plan.rs`, `src/cli/shared.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/backends.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/abstracts.rs`, `src/entities/article/enrichment/tests.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/filters.rs`, `src/entities/article/filters/disease.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/mod.rs`, `src/entities/article/search.rs`, `src/entities/article/search/tests.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/search/tests/finalizer.rs`, `src/entities/article/test_support.rs`, `src/entities/article/variant_search.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/disease/resolution.rs`, `src/entities/disease/resolution/paging.rs`, `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/tests.rs`, `src/entities/drug/approvals.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/label.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/entities/trial/search/nci/tests.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/drug_regulatory.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/mod.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/article_search.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/markdown/test_support.rs`, `src/render/provenance.rs`, `src/render/provenance/drug_regulatory.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/europepmc.rs`, `src/sources/mychem.rs`, `src/sources/opentargets.rs`, `src/sources/pubmed.rs`, `src/transform/article.rs`, `src/transform/article/anchors.rs`, `src/transform/article/federation.rs`, `src/transform/drug.rs`, `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`, `testdata/replay/manifest.json`, `testdata/replay/responses/mydisease/melanoma-detail.json`, `testdata/replay/responses/mydisease/mydisease-query-43314306ecc4.bin`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_article_abstract_recovery.py`, `tests/test_article_disease_filter.py`, `tests/test_article_fulltext_quality.py`, `tests/test_article_metadata_recovery.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_regulatory_replay.py`, `tests/test_drug_target_species.py`.

Drug approvals find Drugs@FDA records filed under active ingredient or brand name, regulatory section status rolls up its parts truthfully, safety labels show their effective date, and an expected empty target answer prints no warning.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R10: `biomcp --json batch drug "daratumumab,belantamab mafodotin" --sections approvals` returns empty approvals while overviews say approved; Drugs@FDA finds daratumumab under products.active_ingredients.name (BLA761036, BLA761145) and BLENREP under products.brand_name (BLA761440); openfda.generic_name finds none. R11: `biomcp --json get drug zotiraciclib regulatory --region us` reports regulatory as data while approvals is empty and orphan designations unavailable. R16: `get drug lorcaserin safety` shows a 2018 label (effective_time 20180429) with nothing marking its age. R15: `get drug --name "trichostatin A" targets` warns "missing field; degrading response" when OpenTargets returns mechanismsOfAction:null and no source holds a target.

## Keeps

Keep the drug identity guards; a fallback match must be the same drug, per `sdlc/decisions/2026-10-19-biologic-approvals-match-the-exact-product.md`: DARZALEX FASPRO (BLA761145) is never attributed to plain daratumumab. Show the label date, not an inferred withdrawal.

## Proof

Replay tests for each of the four cases, plus a negative control showing BLA761145 is not reported as an approval of plain daratumumab.

## Implementation evidence

Fresh developer completed b67528990f8301540f22a49545b2a4e1adc8d259. Meaningful red jobs 2270/1792438868712-18251 and 1792439058302-58762 reproduced approval discovery, aggregate source state, label date and null-response failures after correcting initial fixture setup. Clinical identity controls then exposed the unsafe generic-suffix interpretation and clarified exact product matching. Combination red 1792439761154-60699 demonstrated valid requested combinations being dropped; requested-set red 1792440019426-24849 demonstrated contradictory extra label ingredients being admitted. Final job 1792440059064-28743 passed nine outside replays, 89 affected Rust tests, formatting, warnings-denied Clippy, Ruff and quality checks. Checked final source matches the pushed developer revision.

Approval fallback preserves exact identity from source-owned brand and full ingredient mapping; no generic-suffix normalization remains. Plain daratumumab excludes BLA761145, while an explicitly resolved combination requires complete exact requested, label and product ingredient sets, preserving suffixes and salts. For combinations, reject additional, missing, unnamed or contradictory ingredients and retain only matched whole products. A single product may use an independently established exact brand to rescue missing ingredient metadata, while contradictory or additional actives still refuse. The clinical ruling is recorded beside this ticket. Regulatory aggregation reflects actual source outcomes; label dates remain source dates, and healthy null Open Targets replies remain empty without inventing provider errors. Retained trichostatin stderr contains the mechanismsOfAction warning; its checksum verifies the original null case. No clinical indication, interchangeability or live approval claim is inferred. No full gate is claimed.

Landing integration: the branch contains reviewed 2267 and its runtime prerequisites, plus current main; the dependency records the single batch integration order and adds no product outcome. Source merges were automatic; only changelog and ticket conflicts were resolved by the coordinator. Fresh review covers the final combined drug callers.

Keep: experiments/2091-biomcp-2270 — retain original private research replies, controlled red/green proof, exact checked patch and source hashes.

## Fresh review evidence

Fresh reviewer accepted a78a362738432b731aeb68076154cae9b1edc652 with a scoped source fix. Red 2270/1792440514762-37846 reproduced a false approval from a suffix-derived label brand, including the ingredient-missing rescue route. Source fix 199fa27a0 preserves brands established before label enrichment and requires an exact full label generic or an independently established brand for that link. Green 1792440729849-55589 passed all nine regulatory replays, formatting, warnings-denied Clippy, Ruff and quality checks. Accepted development-code identity and source species behavior remain intact in the combined callers. The synthetic BLENREP fixture now proves an explicit exact generic-to-full-substance source mapping, not suffix-derived equivalence.

Bounded live identity checks 1792440696398-51997 and 1792440712134-54018 found a MyChem exact-name DrugBank hit without independent brand fields, and healthy openFDA BLENREP label 404 replies. Original research output shows Blenrep but does not prove brand ownership. Current live BLENREP discovery remains unproved and is not claimed fixed by the synthetic mapping. No broader biological identity inference was added.

PM disposition: Close with the provider identity limit after explaining successful empty answers. Healthy Drugs@FDA replies with no exact identity match must say that no exact-name match was found; suffixed biologic names remain unverified without source-confirmed identity. No suffix inference or broader matching is authorized. This condition is implemented below.

## Accepted batch repair integration

The coordinator merged accepted 2268 repair 02dc317210c06abeb2b0bb1579c032537285510b plus its review, and accepted 2264 fixture repair 22731420623a21a48d0a863d1f793cf53e6a4e1b plus its review. Both merges were automatic and introduced no conflict resolution or new regulatory source edits. Article full/compact scope and explicit authored NCI absence fixtures are covered by their owning reviews and focused checks. The nine-ticket combined gate owns their final union proof; the PM provider-limit disposition above is now resolved. No redundant regulatory checks ran for those unrelated imports.

## Provider-limit explanation

Developer 0703af568e50eb028eaee62c8eccd42a8fe8f946 adds the successful-empty explanation to JSON and both approvals and regulatory Markdown. Provider failures, disabled and unrequested outcomes retain their existing meaning; matching guards and source calls are unchanged. Red job 2270/1792442474369-89437 failed all three new cases on fe0edec87. Green 1792442597483-35800 passed twelve CLI cases and 34 affected Rust tests, then exposed an unused test-only import in Clippy. Final mechanics job 1792442781009-51784 passed formatting, warnings-denied Clippy, Ruff and quality checks after scoping that import to tests. Retained experiment 2091 owns the exact patches, logs and source comparison. The original live BLENREP discovery remains unverified. No full gate is claimed.
