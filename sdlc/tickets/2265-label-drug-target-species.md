---
priority: 2
---

# 2265 — Label drug target species

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `CHANGELOG.md`, `spec/entity/drug.md`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/mychem.rs`, `src/transform/drug.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_drug_target_species.py`, `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2259-check-replay-recordings-against-live-providers-before-release.md`, `sdlc/tickets/2260-share-one-replay-server-per-test-run.md`, `sdlc/tickets/2261-accept-other-transcript-protein-changes-only-for-the-same-allele.md`, `sdlc/tickets/2262-set-stable-0-9-4-metadata-and-dispositions.md`, `sdlc/tickets/2263-surface-mcp-argument-parse-errors-to-the-model.md`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2266-resolve-drug-development-codes.md`, `sdlc/tickets/2267-enforce-or-document-the-article-disease-filter.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`.

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
