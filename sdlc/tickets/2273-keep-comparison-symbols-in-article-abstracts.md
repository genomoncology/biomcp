---
priority: 1
---

# 2273 — Keep comparison symbols in article abstracts

Status: OPEN.

Milestone: 0.9.4

Depends on: 2271

Reviews: revision 05ee3ca, accept

Reviews: revision 85c2fce, accept

Reviews: revision 99ae3c8, accept

Reviews: revision a622c76, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `spec/entity/drug.md`, `spec/fixtures/setup-article-fulltext-source-fixture.sh`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/article/assets.rs`, `src/entities/article/detail.rs`, `src/entities/article/detail/tests.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/error.rs`, `src/main.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/mychem.rs`, `src/sources/pubmed.rs`, `src/transform/drug.rs`, `tests/article_asset_terminal.rs`, `tests/article_asset_terminal/source_outcomes.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_article_abstract_fidelity.py`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

`get article` abstracts keep every character the publisher's PubMed record holds, including comparison symbols such as ≧, ≥ and ≤, so no threshold is dropped or inverted.

## Evidence

Research (R17, verified against PubMed EFetch): `biomcp --json get article 33311791` renders "Elderly patients ( 65 years, n=91)". PubMed EFetch holds "≧65 years", but PubTator3's biocjson has already dropped the symbol, and BioMCP passes PubTator3's abstract through. A dropped comparison symbol can invert a threshold in clinical text.

## Keeps

Prefer PubMed's abstract text when both are available, or detect characters PubTator3 lost against PubMed's text and use PubMed's. Keep PubTator3's annotations. Coordinate with 2268, which backfills empty abstracts.

## Proof

A replay test for 33311791 shows "≧65 years". A test with symbols ≥, ≤, <, >, ± and µ in the PubMed reply keeps them all.

## Implementation evidence

Fresh developer stopped clean at df6d47654f579459c911caada30d2a1f5d74ec63. Baseline d836b949 and original primary replies reproduced the lost comparison symbol in PMID33311791. Red job 2273/1792437217610-5002 failed the captured-text and symbol checks; its wrapper subsequently failed on a reserved zsh variable, which is retained honestly. Green job 2273/1792437409723-10468 passed six CLI controls, twenty-one targeted Rust tests, formatting, warnings-denied all-target Clippy, Ruff and structural checks. Original provider-wire check 2273/1792437698557-52616 passed three controls. PubMed abstract text replaces nonblank fallback text after shared identifier resolution while PubTator annotations remain. XML text decoding preserves comparison characters without HTML-like stripping. This adds one cached request bounded by the existing ten-second citation limit to detail consumers, with no variant-search work units. Unavailable or empty PubMed preserves existing text and cannot restore unavailable source characters. Full raw replies remain private; no full gate is claimed.

Carried records and milestone declarations are approved PM ancestry.

Keep: experiments/2084-biomcp-2273 — retain primary or authored inputs, baseline, red and green results, commands, tested patches and source hashes.

## Middle-dot decimal proof

The PM added PMID29625879, whose PubTator abstract drops middle dots in survival and hazard-ratio values. Test-only commit a006dbbb266bd3857ac1d557665bdc43484a475b extends the existing CLI regression with numeric XML entities and exact literal middle-dot output while retaining annotations. Product code is unchanged. Red job 2273/1792438752378-70870 made both new controls fail against actual main df231998. Green job 2273/1792438795843-88630 passed four public controls and one private original-wire control plus Ruff and whitespace checks. An earlier adapter-selection mismatch is retained, not represented as a product failure. Reused prepared source files match accepted df6d47654. Current primary replies and hashes remain private under distinct middle-dot names in experiment2084.

Landing integration: this branch contains the reviewed 2271 prerequisite, so the accepted runtime batch can use one final gate. The dependency records that integration order; it adds no product outcome.
