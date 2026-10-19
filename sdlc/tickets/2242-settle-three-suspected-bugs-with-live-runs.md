---
priority: 2
---

# 2242 — Settle three suspected bugs with live runs

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each of three suspected user-facing bugs is either shown fixed on current main or fixed in this ticket.

## Evidence

1. `sdlc/issues/2026-09-28-aspirin-mac-run-never-tested-the-single-ingredient-anchor.md`: `biomcp drug interactions aspirin` may resolve plain aspirin to a five-ingredient combination product and report "not covered".
2. Finding 4 of `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md`: `biomcp discover "H3K27M"` suggested `get drug "h3k27me3"`, a histone mark offered as a drug, and suggestions favor the first synonym over the term typed.
3. `sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md`: a GTR read returned zero rows under its fixture and the spec block was deleted with no replacement.

## Keeps

Fix only what a run shows broken. Clinical judgment about the right drug or entity goes to a clinical reviewer.

## Proof

One live opt-in run per item on current main with synced bundles, recorded. For each broken item, a replay or fixture test that fails first and passes after the fix. Mark each issue resolved with its outcome.

## Coordinator implementation brief

- Changes: `CHANGELOG.md` `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md` `sdlc/issues/2026-09-28-aspirin-mac-run-never-tested-the-single-ingredient-anchor.md` `sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md` `sdlc/tickets/2242-settle-three-suspected-bugs-with-live-runs.md` `spec/entity/diagnostic.md` `spec/fixtures/setup-article-fulltext-source-fixture.sh` `spec/fixtures/setup-provider-contract-spec-fixture.sh` `spec/surface/mcp.md` `src/cli/diagnostic/dispatch.rs` `src/entities/diagnostic/search.rs` `src/entities/discover.rs` `src/transform/drug.rs` `tests/fixtures/live-suspects/aspirin-authored.json` `tests/fixtures/live-suspects/ddinter-authored.csv` `tests/fixtures/live-suspects/discover-authored.json` `tests/fixtures/live-suspects/discovery-controls-authored.json` `tests/fixtures/live-suspects/disease-detail-authored.json` `tests/fixtures/live-suspects/disease-empty-authored.json` `tests/fixtures/live-suspects/disease-query-authored.json` `tests/fixtures/live-suspects/provenance.json` `tests/test_live_suspect_regressions.py`.

Issue closure claim: `sdlc/issues/2026-09-28-aspirin-mac-run-never-tested-the-single-ingredient-anchor.md` `sdlc/issues/2026-09-09-findings-from-the-2026-09-01-interface-study-with-no-ticket.md` `sdlc/issues/2026-10-07-diagnostic-synonym-provenance-fails-spec-contracts.md`.

Restated outcome: preserve the selected aspirin source identity alias; recommend discovery commands only with affirmative entity evidence and preserve the exact entered H3K27M identity; restore deterministic diagnostic synonym coverage and name unavailable resolution in an empty response.

Current-main study on 08d721f8525a35267bcd0a307248c69b1de210fc: jobs 1792421508620-11122 and 1792421728413-59631 retained exact commands, source replies and bundle rows in experiments/785-biomcp-2242-live-suspects. Aspirin kept DB00945/CHEMBL25 but omitted the selected DrugBank name Acetylsalicylic acid from aliases, returning not_in_ddinter_coverage despite DDInter20. H3K27M suggested get drug h3k27me3. Bachmann-Bupp resolved to zero MyDisease matches; the bundle used its expanded condition name, so the empty result does not establish a GTR parser defect. The old fixture still contacted the live resolver.

Clinical ruling: selected single-ingredient source names support aspirin identity, not combination equivalence. ChEBI membership alone does not establish a drug. Require structured affirmative drug classification or supported drug identity, preserve exact NCIT C129310 synonym H3K27M, and do not invent a gene/transcript/genomic allele. Keep unsupported chemical and unclassified vaccine candidates conservative. No loose description keyword inference. Sources: https://www.ebi.ac.uk/ols4/api/ontologies/chebi/terms?obo_id=CHEBI:85045 ; https://api-evsrest.nci.nih.gov/api/v1/concept/ncit/C129310?include=summary ; https://api-evsrest.nci.nih.gov/api/v1/concept/ncit/C180377?include=summary ; https://www.ebi.ac.uk/chembl/api/data/molecule/CHEMBL25.json .

PM permits closing the diagnostic part with deterministic disease-resolution coverage and the recorded upstream gap, without another provider. Include a small source note in JSON and Markdown for unavailable synonym resolution and preserve literal matching. Do not claim the upstream response supplies the expanded name.

Before committing captured public fixtures, inspect source redistribution rules and existing capture standards. Keep complete raw replies privately in the experiment. Do not publish a full DrugBank-containing MyChem reply based only on API accessibility. Use an existing permitted capture or a clearly authored/minimized fixture with truthful provenance; report any unresolved publication question before including that data. Record exact fixture paths in the final manifest.

Write failing outside-in cases first, then minimal fixes. Test only affected cases and mechanical checks through the configured build job, retaining raw evidence outside transient jobs. No additional live reruns requested. No WIP pushes; publish once review-ready.

## Focused repair proof

Source907c1f410287af4ba49651017b7cb5789edb5310 implements the granted identity/discovery/diagnostic changes and eight explicitly authored fixtures, not captured upstream replies. Red1792423169809-80267 had five expected failures and one conservative vaccine pass. Final private-patch1792423468456-55414 passed74focused Rust cases, seven Python checks with two Linux-only skips, eight diagnostic specification blocks, raw MCP JSON/Markdown checks and make lint. An initial compile typo was corrected before green; logs retained. No additional live runs or provider were added. Source licensing review found DrugBank needs a license and upstream rights persist through MyChem; full3.4MBreply remains private and no redistribution permission is inferred. Raw replies, authored fixture provenance, exact commands, patch and manifests remain in experiments/785-biomcp-2242-live-suspects.

Keep: experiments/785-biomcp-2242-live-suspects — Retain cited raw proof until release evidence is explicitly released.
