---
priority: 2
---

# 2267 — Enforce or document the article disease filter

Status: COMPLETE.

Milestone: 0.9.4

Depends on: 2274

Reviews: revision ba395a3, accept

Reviews: revision bd3fbcf, accept

Landed: 995dc68

## Outcome

Changes: `CHANGELOG.md`, `docs/user-guide/cli-reference.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `sdlc/tickets/2274-resolve-common-disease-names-past-the-first-page.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/article/fulltext_view.rs`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/backends.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/article/enrichment.rs`, `src/entities/article/enrichment/abstracts.rs`, `src/entities/article/enrichment/visible_metadata.rs`, `src/entities/article/filters.rs`, `src/entities/article/filters/disease.rs`, `src/entities/article/fulltext.rs`, `src/entities/article/search.rs`, `src/entities/article/search/tests.rs`, `src/entities/article/search/tests/deadline.rs`, `src/entities/article/search/tests/finalizer.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/disease/resolution.rs`, `src/entities/disease/resolution/paging.rs`, `src/entities/disease/resolution/paging_tests.rs`, `src/entities/disease/resolution/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/europepmc.rs`, `src/sources/mychem.rs`, `src/sources/pubmed.rs`, `src/transform/article.rs`, `src/transform/article/anchors.rs`, `src/transform/article/federation.rs`, `src/transform/drug.rs`, `testdata/disease-resolution/melanoma-page-one.json`, `testdata/disease-resolution/melanoma-page-two.json`, `testdata/replay/manifest.json`, `testdata/replay/responses/mydisease/melanoma-detail.json`, `testdata/replay/responses/mydisease/mydisease-query-43314306ecc4.bin`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_article_abstract_recovery.py`, `tests/test_article_disease_filter.py`, `tests/test_article_fulltext_quality.py`, `tests/test_article_metadata_recovery.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

`search article -d` either filters every source's rows to the disease or is documented and labeled as ranking only; help and output agree with the behavior.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R6: `biomcp search article -d "rectal cancer" -k "locally advanced initial staging routine PET CT guidelines" --full --json --limit 5` returns PMID 39118875 (lung cancer, from Semantic Scholar) in third place; its PubMed record contains no "rect". Help calls `-d` a filter.

## Keeps

Prefer enforcing it as a filter on sources that cannot filter upstream, using the disease and its synonyms, if that keeps results useful; otherwise rename it in help as a ranking term. Keep other sources' behavior.

## Proof

A replay test for the rectal cancer query shows the lung cancer row removed, or labeled as unfiltered with help saying so.

Landing integration: this branch contains the reviewed 2274 prerequisite so the runtime batch uses one final gate; this records integration order and adds no product outcome.

## Implementation evidence

Original developer completed 38897fbcfcdb2efdde975f4b879ff1f9a96964f2. Corrected red job 2267/1792437234399-17299 reproduced five intended failures; green 1792437583315-83399 passed seven CLI controls, and 1792438012652-31166 passed four strict goldens, ten affected Rust tests, Clippy and quality checks. Source-backed requested, canonical and exact synonym phrases filter Semantic Scholar and LitSense with token boundaries. Existing upstream-filtered sources retain their behavior. Resolver failure has no literal fallback: semantic source status is unavailable or degraded, complete is false, other source results remain.

Fresh integration developer combined accepted 2274 paging with the filter at 4290750fe60ea4bef75b455dd5b68f154322620e. Job 1792439880079-84943 passed twelve CLI cases before goldens exposed a missing replay detail route. Corrected job 1792439919011-90815 passed all four goldens plus formatting, warnings-denied Clippy, Ruff and quality checks. Actual melanoma capture pages match public fixtures, total 90 over 50 and 40 records. Literal and exact-synonym results remain complete; later-page failures remain incomplete. Fresh reviewer accepted the entire integrated change without fixes or redundant tests. No live full rectal-cancer recall claim or full gate is made. Final release drift inventory must be recomputed after the added captures.

Keep: experiments/2082-biomcp-2267-disease-filter — retain original research rows, controlled red/green and integration proof, checked patches and hashes.
Keep: experiments/2088-biomcp-2274-disease-pages — retain the actual complete disease paging replies and dependency proof.
