---
priority: 2
---

# 2265 — Label drug target species

Status: COMPLETE.

Milestone: 0.9.4

Depends on: 2264

Reviews: revision b8df8ae, accept

Reviews: revision b293fe2, accept

Reviews: revision 6f39103, accept

Reviews: revision 686f218, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `spec/entity/drug.md`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/mychem.rs`, `src/transform/drug.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_target_species.py`, `src/render/markdown/sections/tests.rs`.

`get drug ... targets` labels each target's organism, lists human targets first, and suggests `get gene` only for human targets.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R2: `biomcp --json get drug bortezomib targets` lists PF3D7_1011400 first with no organism and suggests `get gene PF3D7_1011400`; MyChem's gtopdb interaction_targets give species "Plasmodium falciparum 3D7".

## Keeps

Non-human targets stay listed, labeled. Species comes from the source record, never inferred.

## Proof

A replay test for bortezomib shows the species label, human targets first, and no gene suggestion for the Plasmodium target.

## Implementation evidence

Fresh developer stopped clean at 554f6e34b403e710e805a1b02405970c75347a7a. Red job 2265/1792436708312-42916 reproduced parasite-first ordering on the unchanged source. Final job 2265/1792437154389-71122 passed the outside-in replay, sixteen affected Rust tests, formatting, warnings-denied all-target Clippy, Ruff and quality boundaries. Source species remains authoritative; missing species remains unknown, and only source-confirmed human targets receive gene suggestions. Existing target strings remain compatible; additive organism records preserve provenance and share ordering logic. No full gate or separate archival specification run was claimed.

Carried ticket records are approved PM ancestry.

Keep: experiments/2083-biomcp-2265 — retain baseline, red and green results, commands, tested patches, source hashes and permitted private research evidence.

Landing integration: this branch contains the reviewed 2264 prerequisite, so the accepted runtime batch can use one final gate. The dependency records that integration order; it adds no product outcome.

## Discovery fixture integration repair

The eleven-member gate 2264/1792443577982-14890 failed three complete drug command-discovery arrays because the shared fixture supplied ODC1 with no target organism. Accepted runtime behavior correctly withheld that unverified gene suggestion. Developer 38e84b49c5ba249d8b038db8b4d8e7aa86d39140 changes only src/render/markdown/sections/tests.rs: parse existing captured bortezomib data through the existing transform and expect its source-confirmed human PSMB5. Every complete array, recovery case, region exclusion and nine/ten-item boundary remains. The PM approved this source-confirmed fixture swap. No runtime guard or provider response changed.

Focused M5 job 2265/1792444039975-71542 passed twenty affected Rust tests, formatting, warnings-denied Clippy and quality checks. It tested published base cf998b017b7eac2c1db63c08b1c7fb49868ea681 plus verified private delta before the final push; tested and pushed tree 24b3589c5b33e99978fb732e9313f05ebe2ea1a6, delta SHA256 a2f74ea228a9f20a29fc6eee980f140dfac9ea723ff930d551d278cf1215e320. Captured fixture blob 3d6994009b6fd263de01fae7a4dcf726d2d573d3. Existing experiment 2083 retains focused proof; 2080/batch-gate/1792443577982-14890 retains the combined red. All 1569 Python cases passed that red gate; no repeat is claimed. No full gate ran for this repair.
