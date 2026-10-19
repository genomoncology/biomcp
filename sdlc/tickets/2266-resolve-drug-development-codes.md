---
priority: 2
---

# 2266 — Resolve drug development codes

Status: OPEN.

Milestone: 0.9.4

Depends on: 2265

Reviews: revision d382080, accept

Reviews: revision 8458f98, accept

## Outcome

Changes: `CHANGELOG.md`, `sdlc/pm.json`, `sdlc/tickets/2264-report-disease-name-lookup-failures-as-unavailable.md`, `sdlc/tickets/2265-label-drug-target-species.md`, `sdlc/tickets/2268-backfill-missing-article-abstracts.md`, `sdlc/tickets/2269-report-article-metadata-and-section-completeness-truthfully.md`, `sdlc/tickets/2270-fix-drug-approvals-and-regulatory-rollups.md`, `sdlc/tickets/2271-name-the-real-source-for-article-asset-misses.md`, `sdlc/tickets/2272-offer-full-fda-approval-history.md`, `sdlc/tickets/2273-keep-comparison-symbols-in-article-abstracts.md`, `spec/entity/drug.md`, `src/cli/tests/next_commands_json_property/variant_drug.rs`, `src/entities/disease/fallback.rs`, `src/entities/disease/fallback/tests.rs`, `src/entities/drug/cell_lines/tests.rs`, `src/entities/drug/development_codes.rs`, `src/entities/drug/get.rs`, `src/entities/drug/mod.rs`, `src/entities/drug/target_organisms.rs`, `src/entities/drug/targets.rs`, `src/render/json.rs`, `src/render/markdown/drug.rs`, `src/render/markdown/drug/tests.rs`, `src/render/markdown/drug/tests/label_sections.rs`, `src/render/markdown/drug/tests/label_warnings.rs`, `src/render/markdown/evidence/tests.rs`, `src/render/markdown/related.rs`, `src/render/markdown/related/tests/drug_variant_article_trial.rs`, `src/render/markdown/related/tests/gene_drug.rs`, `src/render/markdown/root_tests.rs`, `src/render/provenance.rs`, `src/sources/chembl.rs`, `src/sources/chembl/molecules.rs`, `src/sources/mychem.rs`, `src/transform/drug.rs`, `tests/fixtures/drug/bortezomib-targets-source.txt`, `tests/fixtures/drug/bortezomib-targets.json`, `tests/test_disease_name_outcomes.py`, `tests/test_drug_development_codes.py`, `tests/test_drug_target_species.py`.

`get drug` resolves development codes such as MK-3475 and CT-011 through ChEMBL molecule synonyms when the free-text best hit has no drug identifiers, and single and batch calls give the same answer.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R5: `biomcp --json get drug MK-3475` and `get drug CT-011` return not_found, though ChEMBL lists MK-3475 as a synonym of CHEMBL3137343 and CT-011 of CHEMBL2108680; a batch call once resolved MK-3475.

## Keeps

The drug identity guards (no swapped drugs, brand and ingredient rules) stay. An ambiguous synonym refuses with candidates. Clinical identity questions go to a clinical reviewer.

## Proof

Replay tests resolve both codes in single and batch calls to the same drug; an ambiguous synonym refuses.

## Implementation evidence

Fresh developer completed cfe82e693107261c983ec4514d33aa9c84b567ae. Red jobs 2266/1792438800063-90093 and 1792438810408-93337 reproduce missing development-code resolution on actual main and accepted 2264. Green 1792438941463-40070 passed 11 CLI controls; 1792439018785-54853 passed 98 affected Rust tests and mechanics; exact source job 1792439266097-20812 passed the CLI controls and bounded live identity checks. Live sources confirmed MK-3475 as pembrolizumab CHEMBL3137343 and CT-011 as pidilizumab CHEMBL2108680; single and batch agree. No enrichment or clinical inference changed.

Fresh reviewer found and fixed a qualified-name selection gap: red 1792439619425-24250 reproduced accepting a longer preferred card name despite the expected molecule identifier. Final d382080363f21d94a068dd7249beaa92b041c5fd requires both the exact preferred name and expected identifier. Green 1792439635373-29717 passed seven development-code CLI cases, build, formatting, Ruff and warnings-denied Clippy. Review accepts that final revision; original checked source plus exact fix matches the tested candidate.

Only identifier-free, non-combination, letter-prefix/numeric-suffix codes enter the rescue. Incomplete, ambiguous, substring-only and conflicting identity evidence refuses. Provider failures remain provider failures. No full landing gate is claimed.

Keep: experiments/2089-biomcp-2266-development-codes — retain raw source replies privately, red and green checks, final tested patches and hashes.

Landing integration: this branch contains the reviewed 2265 prerequisite, so the accepted runtime batch can use one final gate. The dependency records that integration order; it adds no product outcome.
