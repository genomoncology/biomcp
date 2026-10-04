# Reconcile maintenance before the next consumer implementation

Date: 2026-10-03. Status: prepared candidate; independent review and shared integration remain pending.

Ian authorizes merging maintenance under the standing programme instructions. The coordinator requested an isolated Quick Fix candidate. This record serves the supported-product preservation requirement in [BioData ADR 0029](https://github.com/genomoncology/biodata/blob/main/sdlc/planning/adr/0029-integrate-biodata-into-the-working-biomcp-product.md). The [2007 reconciliation](2007-maintenance-reconciliation.md) supplies prior classification practice. The current Git merge base supplies the complete interval boundary; the older record's cutoff does not replace it. This Quick Fix creates no product ticket. Ian can overturn the routine compatibility classification.

## Exact inputs and custody

- Development parent: `5b284c6f93697d8d0e625abf665c460820d4fb4e`.
- Last incorporated maintenance ancestor / merge base: `d7f4ee7332747dde5b83aee41f832678af258bc0`.
- Fetched maintenance main, resolved from `8277c521`: `8277c521a4d350203d924b609a09da9ee40239e1`.
- Candidate branch: `ticket/qf-maintenance-20261003`.
- New Git-registered isolated worktree: `biomcp-qf-maintenance-20261003`, created at the exact development parent.
- Incorporation uses `git merge --no-commit --no-ff 8277c521a4d350203d924b609a09da9ee40239e1`, followed by an ordinary two-parent merge commit. No maintenance cherry-picks or rebase.
- The full incoming interval contains 29 commits, including 19 non-merge commits, and 30 changed paths. The effective candidate changes 27 of those paths and adds this record, for 28 paths relative to development.
- Maintenance main, shared development, the retained coding-consumer proof checkout, and the next consumer's frozen inputs are not edited. Shared development remains at the named development parent during preparation.

## Complete interval classification

The release workflow adopts the known architecture correction. Release-date changes remain inapplicable to the existing development citation and changelog policy. Documentation and records preserve the maintenance owner's evidence without implementing its future product plans.

| Incoming path | Treatment |
| --- | --- |
| `.github/workflows/release.yml` | Adopt the Intel wheel-smoke runner pin by merge. |
| `CHANGELOG.md` | Preserve Unreleased and the development history; add described maintenance entries for 1286 and 1287. |
| `CITATION.cff` | Inapplicable release-date correction; retain the development branch citation baseline 0.9.0 / 2026-09-16. |
| `docs/reference/source-licensing-evidence-2026-09-27.md` | Adopt documentation counts and policy-reference explanation by merge; no licensing or source-access configuration changes. |
| `sdlc/issues/2026-09-30-review-of-the-fifth-0.9.1-go-request.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/issues/2026-10-01-review-of-the-sixth-0.9.1-go-request.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/issues/2026-10-02-citation-evidence-masks-its-provider.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/planning/2026-10-03-prove-agent-value.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/records/1221-let-outbound-tls-trust-a-user-supplied-ca-bundle.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/records/1284-close-the-fifth-go-request-round.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/records/1285-the-rehearsal-found-the-ddinter-smoke-gap.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/records/1286-pin-the-x86-64-macos-wheel-smoke-to-an-intel-host.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/records/1287-post-release-sweep.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/release-checklist.md` | Adopt the maintenance owner's checklist by merge as release documentation; authorize no rehearsal, service, or release. |
| `sdlc/tickets/1221-let-outbound-tls-trust-a-user-supplied-ca-bundle.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1284-close-the-fifth-go-request-round.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1285-the-rehearsal-found-the-ddinter-smoke-gap.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1286-pin-the-x86-64-macos-wheel-smoke-to-an-intel-host.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1287-post-release-sweep.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1290-make-the-variant-headline-match-current-clinvar.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1291-reproduce-the-clinvar-section-source-switch.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1292-resolve-intronic-deletion-ranges-in-get-variant.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1293-bound-article-search-time-and-report-partial-sources.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1294-return-whole-abstracts-and-cleaner-full-text.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1295-find-diseases-by-common-abbreviation.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `sdlc/tickets/1296-give-article-entities-identifiers.md` | Adopt by merge as record or planning evidence; no product implementation or new assignment. |
| `tests/test_changelog_coverage_check.py` | Adopt the corrected optional 0.9.1 date by merge; the existing Unreleased alternative remains valid. |
| `tests/test_citation_contract.py` | Inapplicable 0.9 release oracle; preserve the development-version and citation-baseline assertions. |
| `tests/test_docs_changelog_refresh.py` | Inapplicable 0.9.1 release set and heading assertions; preserve the existing development changelog oracle. |
| `tests/test_release_workflow_provenance.py` | Adopt the matching Intel matrix expectation by merge; retain existing development workflow expectations. |

## Preserved behavior and conflict resolution

The four conflict paths are `CHANGELOG.md`, `CITATION.cff`, `tests/test_citation_contract.py`, and `tests/test_docs_changelog_refresh.py`. Resolve only their conflicting development/release hunks. Keep the `Unreleased` heading, all prior development entries, the published citation baseline, and the development test expectations. Add the Intel smoke and release-evidence maintenance descriptions to the existing Internal section. The three conflicted citation/oracle paths remain byte-identical to the development parent. No whole-tree ours strategy is used.

The workflow change pins only the x86_64 installed-wheel smoke host to `macos-15-intel`. The provenance test receives the same expectation. Existing development workflow behavior stays intact. The changelog coverage regex accepts the updated 0.9.1 date while retaining its Unreleased alternative.

Runtime source, CLI/MCP contracts, retrieval, configuration, dependencies, packaging scripts, and producer adoption are byte-identical to the development parent. Rust remains `1.0.0-dev.1`; Python remains `1.0.0.dev1`. The locked BioData dependency remains `ad6219d2e61ed208a8bf6ac4caca1131c4a4e200`. The release workflow is a release automation change, not an application runtime change. Incoming planning and tickets remain evidence, not authorization to implement them.

## Necessary verification and limits

Run the two existing [PM light gates](../pm.json) directly on this candidate: `git diff --check` and `python3 tools/check-tracked-text .`. Also check the staged and complete candidate diff, unresolved markers, changed Markdown local links, exact parent identities, full path classification, development file preservation, and absence of runtime/dependency changes. The link inspection stays local; it does not request external URLs. Historical upstream release runs and producer proof do not establish new combined runtime acceptance.

No broad build, test suite, focused product runner, CI dispatch, release rehearsal, provider call, or service action belongs to this preparation. Push the merge candidate through normal hooks with `[skip ci]`. Independent review must inspect the exact pushed commit and its effective delta before shared integration. No review acceptance, shared landing, release acceptance, or fresh runtime proof is claimed here.

Verification completed on the staged merge candidate on 2026-10-03. `git diff --check`, `git diff --cached --check`, and `python3 tools/check-tracked-text .` exited zero; all five tracked-text scans passed. Local inspection checked both relative Markdown links in the effective changed files and found no missing targets. Exact-base, merge-head, merge-base, unresolved-marker, 28-path envelope, dependency, development metadata, and focused-runner preservation checks passed. The effective delta is 28 files, 27 incoming paths plus this record. Shared development was still clean at `5b284c6f93697d8d0e625abf665c460820d4fb4e` after these checks.

## Independent acceptance and integration

Review: ACCEPT. Checked candidate: `0355281ae24eee66b225cdf92cc338d2064cfb59`. A fresh independent SOL review classified all 30 incoming paths and checked the complete effective merge. Changed matrix and date expectations passed independent source checks, including the failing reverted-host and stale-date alternatives. Runtime source, CLI/MCP contracts, dependencies, producer pin, development versions, citation and prior changelog entries remain unchanged. These source checks establish the bounded maintenance change; they establish no new product runtime result.

The coordinator approves shared development integration under Ian's standing authorization. Existing reviewed product runtime evidence applies only to unchanged effective inputs. The merge adds no feature, release, provider action or hosted check. Final metadata review precedes advancing shared development. Preserve this record's earlier preparation history and the isolated checkout's evidence.

## Amendment, 2026-10-04: current maintenance source preparation

Status: source preparation only; fresh independent read-only review and Root shared integration remain pending. Earlier acceptance above covers its historical candidate only. Root authorizes this amendment under standing maintenance authority and ADR 0029. No duplicate Quick Fix or product ticket is created. Ian can overturn the routine source dispositions.

The clean starting branch was `ticket/qf-maintenance-20261003` at `fdf9a0327d261d1dea0ff8fde084a40fa6aa3ce6`. Local Git objects and remote tips matched the supplied exact revisions. The first normal two-parent merge is `6770e9d3a0c2fc96bbb1d50ee7e614905aa395d5`, with parents fdf9a032 and `c42836e483e3b60d004a179cd09fcb097992dd01`. Its tree is exactly `669776c6f563a7c437596c6b291f96de5f7ce5c4`, the accepted development tree. It was committed and pushed before the second merge. The second normal merge takes that first merge and `d8c6ce8ba495639b8a798e69e58d379d3762b92a` as parents. The complete donor interval starts at `8277c521a4d350203d924b609a09da9ee40239e1`: 57 changed paths, 3,186 inserted lines and 262 removed lines. No rebase, cherry-pick, force push, hook bypass or whole-tree ours strategy is used.

The command-local GitHub CLI credential helper supplies existing authentication for normal HTTPS pushes after the configured keychain helper failed without authentication. Shared configuration was not changed. The effective hooks directory contains only sample hooks; no active commit or push hook was found. Both commits use `[skip ci]`. This preparation initiates no CI or release. Shared development and maintenance source branches remain untouched.

### Every incoming path and behavior

The classifications below cover all 57 paths. Record and planning changes carry the maintenance owner history; their status and QA narratives establish no new combined product proof.

| Incoming path | Classification and specific reason |
| --- | --- |
| `benchmarks/output-footprint/run.py` | adopted by merge: Variant-search ceiling 720 to 800 accounts for source/date provenance bytes; retain every other ceiling and the actual measurement assertions. No benchmark result is claimed. |
| `docs/blog/variant-structure-in-commands.md` | adopted by merge: Label the cached MyVariant RCV summary and show the explicit direct ClinVar follow-up. |
| `docs/sources/clinvar.md` | adopted by merge: Explain direct record-level headline, disagreement and degraded cached fallback. |
| `sdlc/planning/2026-10-03-prove-agent-value.md` | adopted by merge: Carry the maintenance planning edit as historical evidence; grant no new product or experiment work. |
| `sdlc/planning/lanes.md` | adopted by merge: Retain the maintenance draft lane table as evidence; programme PM still selects its existing programme table. |
| `sdlc/pm.json` | inapplicable: The incoming maintenance acceptance list, status mapping, proof cutoff and lane target would replace programme policy. Retain exact development configuration, including targetRef, ownedElsewhere, proof-from-1 and light gates. |
| `sdlc/tickets/1290-make-the-variant-headline-match-current-clinvar.md` | adopted by merge with generic wording correction: Carry the maintenance owner status, design and historical consumer QA evidence with generic unnamed consuming application wording. Preserve evidence values and donor attribution; remove private project names and private repository identifiers. This grants no implementation or combined runtime acceptance. |
| `sdlc/tickets/1291-reproduce-the-clinvar-section-source-switch.md` | adopted by merge: Carry the maintenance owner status, design and QA narrative as historical evidence only; this grants no implementation or combined runtime acceptance for that ticket. |
| `sdlc/tickets/1292-resolve-intronic-deletion-ranges-in-get-variant.md` | adopted by merge with generic wording correction: Carry the maintenance owner status, design and historical consumer QA evidence with generic unnamed consuming application wording. Preserve evidence values and donor attribution; remove private project names and private repository identifiers. This grants no implementation or combined runtime acceptance. |
| `sdlc/tickets/1293-bound-article-search-time-and-report-partial-sources.md` | adopted by merge with generic wording correction: Carry the maintenance owner status, design and historical consumer QA evidence with generic unnamed consuming application wording. Preserve evidence values and donor attribution; remove private project names and private repository identifiers. This grants no implementation or combined runtime acceptance. |
| `sdlc/tickets/1294-return-whole-abstracts-and-cleaner-full-text.md` | adopted by merge: Carry the maintenance owner status, design and QA narrative as historical evidence only; this grants no implementation or combined runtime acceptance for that ticket. |
| `sdlc/tickets/1295-find-diseases-by-common-abbreviation.md` | adopted by merge: Carry the maintenance owner status, design and QA narrative as historical evidence only; this grants no implementation or combined runtime acceptance for that ticket. |
| `sdlc/tickets/1296-give-article-entities-identifiers.md` | adopted by merge: Carry the maintenance owner status, design and QA narrative as historical evidence only; this grants no implementation or combined runtime acceptance for that ticket. |
| `sdlc/tickets/1297-a-protein-change-query-must-not-resolve-to-the-wrong-variant.md` | adopted by merge with generic wording correction: Carry the maintenance owner status, design and historical consumer QA evidence with generic unnamed consuming application wording. Preserve evidence values and donor attribution; remove private project names and private repository identifiers. This grants no implementation or combined runtime acceptance. |
| `skills/schemas/variant.json` | adopted by merge: Declare optional significance source, evaluation date and note fields without removing existing fields. |
| `skills/use-cases/05-variant-pathogenicity.md` | adopted by merge: Request direct ClinVar and distinguish record-level evidence from cached RCV details. |
| `spec/entity/article.md` | adopted by merge: Add the local held-source whole-deadline contract with partial rows and full diagnostics. |
| `spec/entity/variant.md` | adopted by merge: Add recorded cached/direct disagreement, JSON provenance and Markdown headline/record-level contracts; retain all prior cases. |
| `spec/fixtures/cleanup-article-search-deadline-fixture.sh` | adopted by merge: Use existing routine owner cleanup for the new local deadline fixture; not executed. |
| `spec/fixtures/run-article-search-deadline-search.sh` | adopted by merge: Prepare owned fixture through existing setup/cleanup and prepared CLI wrapper; not executed. |
| `spec/fixtures/setup-article-search-deadline-fixture.sh` | adopted by merge: Local provider-shaped handlers hold Europe PMC while other legs answer, with owner/supervisor lifecycle and an eight-second fixture deadline; not executed. |
| `spec/fixtures/setup-variant-identity-spec-fixture.sh` | adopted by merge: Serve the two recorded variant captures and local ClinVar endpoint without changing prior routes. |
| `src/cli/article/dispatch.rs` | adopted by merge: Carry page timings and deadline to full JSON only; compact diagnostics stay absent. |
| `src/cli/article/tests/diagnostics.rs` | adopted by merge: Assert exact full diagnostics and compact omission; no assertions removed. |
| `src/cli/article/tests/exact_lookup.rs` | adopted by merge: Initialize diagnostics on existing test pages; preserve exact-lookup and degraded retry assertions. |
| `src/cli/article/tests/filters.rs` | adopted by merge: Initialize diagnostics on coverage test pages; preserve filter assertions. |
| `src/cli/article/tests/json.rs` | adopted by merge: Initialize diagnostics on existing JSON test pages; preserve context, warnings, next commands and envelopes. |
| `src/cli/article/tests/mod.rs` | adopted by merge: Register the incoming diagnostics module without altering prior modules. |
| `src/cli/article/tests/next_commands.rs` | adopted by merge: Initialize diagnostics while retaining exact-variant follow-up assertion. |
| `src/entities/article/enrichment.rs` | adopted by merge with bounded deadline correction: After in-flight plain Semantic Scholar batch expiry, emit degraded deadline status. After final plain metadata fallback completion or failure, retain materialized rows and name the affected PubTator/Europe PMC chain. Explicit migrated execution remains unchanged. |
| `src/entities/article/mod.rs` | adopted by merge: Add diagnostics and timing records to search pages; preserve migrated article types. |
| `src/entities/article/search.rs` | adopted by merge with bounded deadline correction: Re-export existing deadline detector and mapper within article ownership for the bounded plain-backend correction; preserve deadline orchestration. |
| `src/entities/article/search/deadline.rs` | adopted by merge with bounded deadline correction: Widen only the existing detector and mapper visibility within article ownership; preserve wrapped-error recognition and retryable deadline mapping. |
| `src/entities/article/search/tests.rs` | adopted by merge: Register deadline tests; retain prior acquisition, pagination, retraction, type-capable and merge tests. |
| `src/entities/article/search/tests/deadline.rs` | adopted by merge with bounded deadline correction: Append three source-only signal-held regressions for explicit S2 zero-answer expiry, in-flight batch expiry and final metadata fallback expiry. Each includes successful empty/null control and actual fixture request assertions; retain all existing regressions. |
| `src/entities/article/test_support.rs` | adopted by merge: Add signal-held fixture reply while retaining existing migrated fixture and TestEnv scheduling. |
| `src/entities/variant/get.rs` | adopted by merge: Add direct record-level headline helper and empty literal fields; preserve genomic prediction preparation and every existing migrated getter boundary. |
| `src/entities/variant/get/tests.rs` | adopted by merge: Combine all three incoming headline tests and literal fields with both existing genomic preparation functions. No oracle changes or assertion removal. |
| `src/entities/variant/mod.rs` | adopted by merge: Add optional headline/search provenance and bounded ClinVar record classification; apply it only on usable direct records, retaining existing section outcomes and interval identity. |
| `src/entities/variant/search/tests.rs` | adopted by merge: Six provenance literal lines only; retain exact aggregation, annotation, interval and coding behavior assertions. |
| `src/render/markdown/variant.rs` | adopted by merge: Pass headline source/date/note into the existing safe render context. |
| `src/render/markdown/variant/tests.rs` | adopted by merge: Initialize optional source/date fields in four search literals; preserve all exact transcript/render assertions. |
| `src/render/provenance.rs` | adopted by merge: Initialize three headline fields in the retained test literal; migrated runtime provenance remains unchanged. |
| `src/sources/ncbi_efetch.rs` | adopted by merge: Decode record-level germline classification with budget charges and two recorded/absence regressions; preserve direct ClinVar guardrails. |
| `src/transform/variant.rs` | adopted by merge: Label cached significance as MyVariant with newest RCV evaluation date and direct follow-up note; retain migrated identity/annotation selection. |
| `templates/variant.md.j2` | adopted by merge: Render headline provenance, disagreement/no-classification note and direct germline record detail. |
| `templates/variant_search.md.j2` | adopted by merge: Keep the variant table and transcript explanation slot; add a cached-significance/direct-follow-up explanation. |
| `testdata/sources/capture-receipts.json` | adopted by merge: Retain all development receipts and add two donor public capture provenance entries; no acquisition or provider freshness claim. |
| `testdata/sources/myvariant/search_tp53_g105s_20261003.json` | adopted by merge: Retain exact donor capture bytes and receipt hash as a fixture input; no new provider request. |
| `testdata/sources/ncbi_efetch/clinvar_428884_20261003.xml` | adopted by merge: Retain exact donor XML and receipt hash, including its whitespace-only line; no new provider request. |
| `tests/article_cli_tests_structure.rs` | adopted by merge: Add new CLI diagnostics owner to the exact file-set oracle; no exclusions. |
| `tests/surface/test_source_configuration_docs_contract.py` | adopted by merge with forward correction: Add deadline seam to the existing allowlist. Correct its description to read in release builds because source reads it unconditionally; retain all environment-surface checks. |
| `tests/test_article_spec_fixture_lifecycle.py` | adopted by merge: Add only the owned deadline fixture to the exact unpaced-origin oracle; preserve all ownership checks. |
| `tests/test_source_package_boundary.py` | reimplemented through the BioData boundary: Require all six added package members using the development inclusion oracle. Retain all BioData/site/dependency checks; maintenance zero-coupling/removal rules and numerical package ceiling are inapplicable to the migrated package. |
| `tests/unit/cli/variant.rs` | adopted by merge: Initialize source/date fields in the existing renderer envelope literal; retain assertions. |
| `tools/rust-source-size-inventory.json` | reconciled through the migrated boundary: Retain 49 development objects, floors and conditions; reconcile six changed baselines and add two newly overthreshold retained variant owners. Do not resurrect three retired donor objects or drop typed MCP ownership. |
| `tools/zero-coupling-historical.json` | already satisfied in development: Absent since development restored the BioData boundary. Reject merge resurrection of the retired ledger; no candidate baseline path is deleted. |

### All five conflict paths and hunk decisions

- `sdlc/pm.json`, add/add, two blocks: retain the complete programme configuration because the donor maintenance configuration belongs to another owner. Mailroom and record naming already agree. No incoming acceptance list, maintenance proof cutoff, forbidden-name list or maintenance lane target supersedes programme policy. The final file is byte-identical to c428.
- `src/entities/variant/get/tests.rs`, adjacent additions: close the retained genomic optional-table function, then append all three donor headline functions. Retain the default genomic boundary helper, its optional-feature test, all existing assertions and all incoming assertions. The six added literal-field lines are also retained.
- `tests/test_source_package_boundary.py`, ownership block: preserve every development BioData, website, root-entry, area and dependency assertion. The maintenance ban on the BioData crate and retired zero-coupling checker would contradict the migrated package. Translate the six incoming packaged-path additions into six required-members assertions. The maintenance-only numerical package ceiling has already been replaced by explicit inclusion and boundary proof on development; no development oracle is removed or weakened.
- `tools/rust-source-size-inventory.json`, five conflicting blocks and the sixth auto-merged owner: preserve development floors, owners, reasons, conditions and retired-owner dispositions; add the exact observed source growth and maintenance reasons. Combine the getter preparation allowance with the headline allowance, retain the migrated provenance floor, and preserve programme package-neutral decomposition conditions instead of restoring the maintenance-only package ceiling. The auto-merged variant-transform reason remains development-owned with an appended maintenance explanation. Two additional combined-tree owners need new entries, as detailed below.
- `tools/zero-coupling-historical.json`, modify/delete: retain its preexisting development absence. The maintenance edit only refreshes a receipt hash inside a retired ledger. Rejecting its resurrection introduces no deletion relative to the accepted development tree and changes no active checker.

All auto-merged overlaps were inspected for retained ownership: article enrichment keeps its migrated resolution functions and explicit variant context, test support keeps scheduling, the getter keeps genomic preparation, variant types keep interval identity, search tests keep coding/interval assertions, runtime provenance keeps migrated behavior, and capture receipts keep all development entries. The only additional donor correction changes the article-deadline allowlist description from debug-only to release-read; the source reads this fixture seam unconditionally. It changes no assertion or environment admission.

### Exact effective delta and preserved inputs

Before amendment metadata, the effective delta against c428 is 55 paths: 3,007 inserted lines and 260 removed lines. Categories are 23 Rust paths under src, 10 SDLC paths, 6 spec/fixture paths, 5 other test paths, 3 capture/receipt paths, 2 docs, 2 skill/schema paths, 2 templates, 1 benchmark input and 1 inventory. These are 45 non-SDLC effective inputs and 10 maintenance administrative inputs. The owner record amendment and its machine-readable companion add two changed metadata paths, so the complete candidate delta contains 57 paths. There are no tracked deletions or mode changes. Tracked paths grow from 3,980 to 3,991, including the new companion; src Rust grows from 800 to 803.

The [machine-readable companion](2026-10-04-qf-maintenance-source-inputs.json) seals every changed effective input with old/new byte hashes, modes, Git blobs and exact line deltas. It excludes its own bytes and this record from that input list to avoid recursive seals; the final commit/tree binds both metadata files. It also contains all 51 candidate overthreshold objects with actual source hashes/counts and old/donor/candidate entry dispositions.

Manifest, lock, development version/citation/changelog policy, programme PM configuration, quality checker, accepted producer `ad6219d2e61ed208a8bf6ac4caca1131c4a4e200`, focused runner configuration and existing migration oracle bytes are unchanged from c428. The four coding gold inputs and interval-search oracle remain unchanged. No 0680 CASE artifact or gold is edited. Runtime source changes mean the prior c428 acceptance cannot be relabeled as acceptance of this candidate.

### Complete static source inventory reconciliation

The candidate has 51 overthreshold src Rust files and exactly 51 inventory objects at the unchanged 1,000-line threshold. Static splitlines/non-whitespace counts reconcile every object, exact baseline and delta; this does not execute or qualify the quality checker. Of 49 development objects, 43 are byte-equivalent objects and six gain exact count/reason increments. Every existing floor and removal condition stays exact. The two new entries retain the development source baseline as their floor and name Root maintenance authority, retained assertions and a decomposition/removal condition.

| Changed inventory owner | Development total | Candidate total | Floor | Exact authorized delta |
| --- | --- | --- | --- | --- |
| `src/entities/article/mod.rs` | 1385 | 1406 | 1314 | 92 |
| `src/entities/variant/get.rs` | 1268 | 1307 | 1128 | 179 |
| `src/entities/variant/get/tests.rs` | 911 | 1019 | 911 | 108 |
| `src/entities/variant/mod.rs` | 978 | 1003 | 978 | 25 |
| `src/render/markdown/variant/tests.rs` | 1098 | 1106 | 1000 | 106 |
| `src/render/provenance.rs` | 1863 | 1866 | 1745 | 121 |
| `src/sources/ncbi_efetch.rs` | 1217 | 1281 | 1000 | 281 |
| `src/transform/variant.rs` | 1385 | 1424 | 1335 | 89 |

Donor inventory count 51 is not the candidate inventory count by copying: donor-only retired owners are `src/entities/drug/get.rs`, `src/sources/tests/provider_network.rs` and `src/transform/drug.rs`. Their candidate totals are 661, 956, 771 in that listed order. They remain below threshold. The development-only `src/mcp/shell/typed_get_tests.rs` object remains exact. Together with the two new combined variant owners, this yields 51. The companion records every donor-only object and the measured candidate count. Inventory SHA-256 is `83ddeed0411719cc6e4dadbf7ca47a35ffe34fa945c6836784abe20366cb099c`.

### Nine new source baselines for the later 0681 SOURCE amendment

Counts use splitlines for total and non-whitespace splitlines for nonblank, including comments. The old accepted CASE and its caps are preserved. These observations propose new SOURCE inputs; they do not approve new caps or change the eight-path 0681 claim. Search tests grow by six literal-field lines, from 831/876 to 837/882, leaving 13 nonblank and 18 total lines under the old 850/900 final cap. The frozen getter grows from 1195/1268 to 1233/1307. Its byte-identical requirement must be rebound to the independently accepted new base before coding; seven other frozen/mutable baseline files remain unchanged.

| Path | Candidate nonblank / total | SHA-256 |
| --- | --- | --- |
| `src/entities/variant/resolution/coding_alias.rs` | 181 / 194 | `502262e30683be3c1b9a18973b67158ba756917e7435fd1d89ac9d5acb3e2c28` |
| `src/entities/variant/resolution/coding_tests.rs` | 387 / 402 | `ba2add2fb823012a2aef02e41ba267a0e68fa13a77f033f36b041750efbbd5d9` |
| `src/entities/variant/resolution/coding_transport_tests.rs` | 176 / 179 | `1acd2e67e73ce79575ac9c40a2405af8e4870d945c770ff4ff415c430033260a` |
| `src/entities/variant/search/tests.rs` | 837 / 882 | `f4f2cf93fcacbb643a21455e4c8eab803ac771fc6eb636d81d57cf0d3b55a472` |
| `src/entities/variant/resolution.rs` | 1149 / 1224 | `98cf7b7051c712c88aa9c131dca197ec07caca6c5c98931863aa3c193c5895a6` |
| `src/entities/variant/resolution/tests.rs` | 508 / 548 | `d2b8a5ac3f9efacd948aaac862e3bb94b32b2dc049309f75f522bc370cf1a684` |
| `src/entities/variant/search/mod.rs` | 1069 / 1113 | `a873265c6d2225c71a80edb1f3b68aa3bdb7cc9ef3fc481b9432fa908433c70f` |
| `src/entities/variant/get.rs` | 1233 / 1307 | `e9face3a0208f3941efdb86b3026ddaf2bec4a9f520defb14f3762908b3f4b70` |
| `src/mcp/shell/typed_get_tests.rs` | 1562 / 1631 | `c5bf1f2d30c4f44e4d3918f28ba0494ab12b04af55ee2040f42e26aad894d89c` |

### Finite later OFFLINE proposal and proof boundaries

Propose the 28 literal Rust selectors below as the bounded QF source-dependent proof set. Each name was found in its actual source owner. Choose one existing test binary under no-default-features in a separately reviewed offline packet; do not duplicate the main-source tests across aliases. This is a minimal table-oriented coverage proposal for the changed contracts and migrated boundaries, not collection, execution, acceptance, or permission to run it. It does not replace the fixed 35 selectors or any gold row of 0681. The affected prior article selectors retain no-provider enrichment, raw auxiliary rows and materialized variant work after deadline expiry.

Coverage: new article deadline and diagnostics.

- `entities::article::search::tests::deadline::overall_deadline_returns_partial_rows_and_names_the_held_source`
- `entities::article::search::tests::deadline::failed_primaries_still_return_answered_rows`
- `entities::article::search::tests::deadline::healthy_federated_search_keeps_output_shape_and_records_timings`
- `cli::article::tests::diagnostics::article_search_diagnostics_render_only_in_full_detail`

Coverage: variant headline fallback and provenance.

- `entities::variant::get::tests::headline_follows_record_level_germline_classification_and_names_ncbi`
- `entities::variant::get::tests::record_without_germline_classification_keeps_derived_value_labeled`
- `entities::variant::get::tests::agreeing_record_level_classification_drops_the_cached_copy_note`
- `entities::variant::get::tests::indirect_clinvar_fallback_preserves_accession_freshness_and_submitter_count`
- `sources::ncbi_efetch::clinvar::tests::parses_record_level_germline_classification_from_recorded_tp53_record`
- `sources::ncbi_efetch::clinvar::tests::record_level_germline_classification_is_absent_when_the_record_has_none`
- `entities::variant::clinvar::tests::canonical_outcomes_and_provenance_match_selected_payload_source`
- `render::markdown::variant::tests::variant_markdown_renders_compact_clinvar_and_population_fields`
- `render::markdown::variant::tests::variant_search_explains_distinct_transcript_match_after_unchanged_table`

Coverage: retained migrated resolution.

- `entities::variant::resolution::tests::coding::coding_assertion_resources_and_privacy_table`
- `entities::variant::resolution::tests::coding::coding_identity_comparison_table`
- `entities::variant::resolution::tests::coding::coding_source_projection_table`
- `entities::variant::resolution::tests::coding::transport::coding_cli_and_mcp_table`
- `entities::variant::resolution::tests::genomic::genomic_assertion_and_resource_table`
- `entities::variant::resolution::tests::genomic::genomic_identity_comparison_table`
- `entities::variant::resolution::tests::genomic::genomic_consumer_boundary_table`
- `entities::variant::resolution::tests::interval::interval_alias_pair_resource_privacy_table`
- `entities::variant::resolution::tests::interval::interval_identity_comparison_table`
- `entities::variant::resolution::tests::interval_search::interval_search_adapter_table`
- `cli::variant::interval_search_tests::interval_search_routing_table`
- `entities::variant::search::tests::interval_annotation_comparison_table`

Coverage: affected retained article behavior.

- `entities::article::enrichment::tests::empty_enrichment_plan_makes_no_provider_requests`
- `entities::article::search::tests::raw_federated_acquisition_retains_auxiliary_rows_when_primary_sources_fail`
- `entities::article::variant_search::tests::mid_route_expiry_preserves_materialized_units_across_strategy_identity_matrix`

The source-named public process contract proposal is the two existing pages `spec/entity/article.md` and `spec/entity/variant.md`, including their new deadline/full-diagnostics and cached/direct headline JSON/Markdown cases. Their prepared executable, local fixture and offline runner packet needs separate COMMAND review before execution. Do not invent section filtering that the runner does not support. Existing footprint assertions remain a later separate measurement obligation for the changed provenance byte envelope. No full-suite execution or new helper is proposed.

Risks: the combined source has not compiled; selector registration, fixture lifecycle, timings, cancellation/partial-page behavior, direct ClinVar absence/fallback and template output remain unproved. The migrated variant-article path shares deadline/timing acquisition seams while retaining its explicit execution context, so its unchanged source still needs affected proof. The two newly oversized retained files and six updated source baselines need independent SOURCE acceptance and the unchanged actual audit before integration; static exactness is not audit acceptance. The public recorded captures establish fixture provenance only, not current-provider truth. The donor XML has trailing whitespace on line 2; the normal complete diff check reports it, and exact donor/receipt bytes are preserved. No whitespace rule or test is relaxed to turn that finding green.

Permitted verification: Git object/ancestry/tree inspection, complete donor/effective diff and tracked-mode inspection, static counts/hash/inventory comparisons, source-name inspection, unresolved-marker checks, local relative-link inspection, `git diff --check`, `git diff --cached --check`, and the tracked-text checker. No cargo, pytest, fixtures, quality checker, build, provider, Docker, CI or release action is performed. Root owns later independent read-only review, SOURCE amendment acceptance, offline COMMAND/ACTUAL proof and shared advancement. This writer stops at the clean normally pushed candidate and grants no runtime or release acceptance.

Verification completed on the staged source candidate on 2026-10-04. All five tracked-text scans passed. Static seals, all 51 overthreshold counts/baselines, inherited floors and removal conditions, the 55-input inventory, nine baseline hashes, donor capture receipt hashes, named preserved development inputs, unresolved-marker/index checks and local relative Markdown links passed inspection. The complete c428-relative delta is 57 paths, 6,661 inserted lines and 260 removed lines, including amendment metadata; 11 paths are added and 46 modified. Normal staged/full whitespace checks exit 2 solely for the exact donor ClinVar XML line 2; the unstaged diff check exits zero. This is an explicit unresolved hygiene finding, not a green gate claim or a change to the fixture oracle. The independent reviewer and Root must assess it before shared integration. No runtime or quality-audit result is claimed.

### Forward correction, 2026-10-04: public consumer evidence wording

Root requires generic public wording before handoff. The queued private wording request was not applied in the prior source-preparation turn at `cc26694a9cae8536f8c545075418870165809de3`. This normal forward correction changes only tickets 1290, 1292, 1293 and 1297 plus this existing record and its existing companion. It replaces private project, QA and repository references with historical consumer QA evidence for an unnamed consuming application, attributed to the maintenance owner. It retains report numbers, evidence values, dates, public BioMCP branch revisions, commands and observed outcomes; private repository identifiers are removed. All four donor classifications now say adopted by merge with generic wording correction. The companion refreshes the four candidate byte seals and Git blobs against the original c428 baseline. No duplicate ticket or record is created. BioData 0681 remains a ticket.

The independent review running against fixed cc26694a covers that earlier revision only. This correction requires review against the new exact HEAD/tree before Root advances shared development. Runtime source, all 51 source inventory objects, the nine source baselines, selector proposal, gold, dependency, configuration and retired ledger absence remain unchanged. There is no deletion, cleanup, build, runtime, fixture, provider, Docker, CI or release action, and no Git configuration change.

Final correction verification: the six-path forward delta is 6 paths, 50 inserted lines and 32 removed lines. The complete c428-relative candidate delta is 57 paths, 6679 inserted lines and 260 removed lines, still 57 paths with 11 added and 46 modified, no deleted paths or mode changes. The 55 effective changed inputs remain the same paths; their refreshed total line delta is 3007 inserted lines and 260 removed lines. All five tracked-text scans passed. The staged correction whitespace check passes; the complete candidate whitespace check retains exit 2 solely for the byte-faithful donor ClinVar XML line 2. Exact source-input seals and absence of private project/repository wording in the four corrected tickets passed static inspection. These results establish source/text custody only; no runtime, quality-audit or independent acceptance result is claimed.

### Root source disposition, 2026-10-04: article deadline review findings

Fresh SOL High read-only review `01a107c9-97cd-78e3-bb9e-7e5412a98c98` returned FINDINGS for exact candidate `cc26694a9cae8536f8c545075418870165809de3`. Root authorizes bounded source remediation in the adopted article deadline/partial diagnostics scope. The correction starts from `25161d7546d3c01fcbb9d494c3dc44446495d589`, whose public wording correction remains intact. This record keeps the complete 57-path donor classification. The new extra path `src/entities/article/backends.rs` is separately classified as a bounded forward source correction; it is not an unclassified donor path or a new ticket.

Finding 1: the Semantic Scholar candidate boundary converted request failure into a successful empty/unavailable outcome. An explicit search therefore bypassed the outer retryable deadline mapping. Preserve identifiable invocation deadline errors, including source-context wrappers, when there is no explicit variant execution context. Source inspection also found that the existing Semantic Scholar client sanitizes some outbound errors before this boundary. Map an actual request failure under an exhausted plain invocation to the existing retryable article-search deadline error. Keep ordinary unavailable outcomes, successful empty results and explicit migrated execution behavior unchanged. Re-export only the existing deadline detector and mapper within article ownership; widen their visibility only to that ownership boundary. No client or dependency source is changed.

Finding 2: a failed in-flight Semantic Scholar enrichment batch always reported generic unavailable, and the final metadata fallback row had no subsequent loop iteration to report expiry. On plain batch expiry, emit degraded Semantic Scholar deadline status while retaining previously merged rows. After each plain metadata fallback result, merge any materialized article first, then report the affected PubTator/Europe PMC chain on invocation expiry and stop new row work. This post-operation check covers the final row and a materialized PubTator detail whose optional Europe PMC enrichment failed internally. The explicit variant execution branch remains unchanged and returns before this plain-path status check. Timely successful empty/null responses do not become deadline failures.

| Forward source path | Disposition |
| --- | --- |
| `src/entities/article/backends.rs` | New bounded forward correction under Root authority after review 01a107c9-97cd-78e3-bb9e-7e5412a98c98: preserve plain-search wrapped deadline errors and map sanitized request failures on invocation expiry to retryable SourceUnavailable. Keep successful empty, ordinary unavailable and explicit migrated execution outcomes unchanged. |
| `src/entities/article/enrichment.rs` | After in-flight plain Semantic Scholar batch expiry, emit degraded deadline status. After final plain metadata fallback completion or failure, retain materialized rows and name the affected PubTator/Europe PMC chain. Explicit migrated execution remains unchanged. |
| `src/entities/article/search.rs` | Re-export existing deadline detector and mapper within article ownership for the bounded plain-backend correction; preserve deadline orchestration. |
| `src/entities/article/search/deadline.rs` | Widen only the existing detector and mapper visibility within article ownership; preserve wrapped-error recognition and retryable deadline mapping. |
| `src/entities/article/search/tests/deadline.rs` | Append three source-only signal-held regressions for explicit S2 zero-answer expiry, in-flight batch expiry and final metadata fallback expiry. Each includes successful empty/null control and actual fixture request assertions; retain all existing regressions. |

Exactly three new deterministic fixture selectors are appended in the existing `src/entities/article/search/tests/deadline.rs` owner. Signal-held replies remain blocked until the owned sender is dropped; each attempted operation uses a finite four-second invocation deadline and an existing watchdog. No sleep-driven source release is introduced. The explicit search test asserts retryable zero-answer expiry and successful empty search, and checks wrapped-error detection without treating HTTP 503 as a deadline. The batch test asserts retained row bytes plus degraded status and successful null enrichment. The final-row test observes PubTator lag followed by held Europe PMC, asserts both affected source diagnostics and retained row bytes, and retains the timely empty PubTator detail control. Existing tests and assertions are not removed. None of these selectors or fixtures has executed.

Add these three actual product Rust names to the retained 28-name future proposal, yielding 31 product Rust selectors:

- `entities::article::search::tests::deadline::explicit_semantic_scholar_deadline_is_unavailable_but_successful_empty_is_not`
- `entities::article::search::tests::deadline::in_flight_semantic_scholar_enrichment_deadline_retains_rows_and_names_source`
- `entities::article::search::tests::deadline::final_metadata_fallback_deadline_retains_row_and_names_both_consulted_sources`

Add only these six existing support selectors to the separately reviewed future offline packet. The three Python node names cover affected environment classification, owned fixture admission and package inclusion. The three Rust integration names cover the CLI article test layout. These are literal source names and proposed selections, not collected or executed commands:

- `tests/surface/test_source_configuration_docs_contract.py::test_biomcp_env_docs_match_runtime_reads` in `tests/surface/test_source_configuration_docs_contract.py`.
- `tests/test_article_spec_fixture_lifecycle.py::test_only_owned_article_fixtures_export_unpaced_origin` in `tests/test_article_spec_fixture_lifecycle.py`.
- `tests/test_source_package_boundary.py::test_cargo_source_package_keeps_the_runtime_boundary` in `tests/test_source_package_boundary.py`.
- `article_cli_flat_test_sidecar_is_replaced_by_directory` in `tests/article_cli_tests_structure.rs`.
- `article_cli_test_split_files_exist_with_doc_headers` in `tests/article_cli_tests_structure.rs`.
- `article_cli_test_sidecar_files_stay_under_700_lines` in `tests/article_cli_tests_structure.rs`.

Retain both public spec pages. Retain the unchanged actual `rust_source_size` audit and footprint measurement obligations without execution or waiver. This adds no complete suite. The 0681 ticket's fixed 35-selector CASE and gold remain separate and unchanged.

The candidate now has 56 effective changed input paths before amendment metadata, including 24 changed Rust paths under src and 46 non-SDLC inputs. Backends is the only newly changed path relative to the prior candidate. The complete c428-relative delta therefore has 58 paths: 47 modified, 11 added, no path deletions or mode changes. The checkout still has 3,991 tracked paths and 803 src Rust files. Source growth in this correction does not create another overthreshold file: the five affected source totals are shown below. All 51 candidate inventory objects, floors, conditions and inventoried source hashes remain exact. All nine current source baseline hashes/counts remain exact to the post-maintenance candidate; the old c428 nine-count/49-object inventory cannot be substituted for them. The companion refreshes the complete 56-input seals/deltas and records all 24 changed src Rust counts/hashes.

| Corrected Rust source | Nonblank / total | SHA-256 |
| --- | --- | --- |
| `src/entities/article/backends.rs` | 922 / 975 | `f983a6c3d77bf5ce01f682830d95c29c1b6f3e2306e5026099c5b0020739524a` |
| `src/entities/article/enrichment.rs` | 481 / 510 | `c9ac54c4240e59303b50e074369c883724ff4ac6281dbe3e45ae8186042a0817` |
| `src/entities/article/search.rs` | 926 / 969 | `c8f3027c50618b78e32c1c0725b52be69747e7bba603f955d19ba8c76ff8369f` |
| `src/entities/article/search/deadline.rs` | 107 / 115 | `e7c96897c5a744c09c167045365d6993a96aef9ae60140c57719663f75ba7c4c` |
| `src/entities/article/search/tests/deadline.rs` | 402 / 422 | `3ac6fcd7ab51aa451798f18b74aa29042c42a58ee6c76d116f439e2903728f16` |

Source-only verification: static source-name, exact input hash/mode, retained inventory/baseline, donor classification, gold/producer/configuration, private-wording, unresolved-marker and local-link inspections passed. All five tracked-text scans passed. The staged correction whitespace check passes. The complete candidate whitespace check retains exit 2 solely for the byte-faithful donor ClinVar XML line 2; the original XML bytes are unchanged. No quality checker, formatter, compiler, build, runtime, fixture, provider, Docker, CI, release or deletion action is executed. No Git configuration changes or hook bypass are used. The forward delta against 25161d75 is 7 paths, 633 inserted lines and 39 removed lines; the complete c428-relative delta is 58 paths, 7275 inserted lines and 262 removed lines. Effective inputs excluding amendment metadata account for 3232 inserted lines and 262 removed lines.

Risks and next owner: tests are authored but uncompiled and unexecuted. Source inspection cannot establish fixture routing, request registration, timing settlement, cancellation behavior, retained page output or successful controls at runtime. The metadata diagnostics describe the affected fallback chain, not a claim that both provider requests independently hung. The existing Semantic Scholar sanitization stays intact outside this plain invocation boundary. Root obtains fresh independent source review of the new exact pushed HEAD/tree, updates the 0681 SOURCE amendment, and separately owns offline COMMAND/ACTUAL acceptance, actual audit, footprint evidence and shared integration. Earlier review or retained runtime evidence supplies no acceptance for these changed effective inputs.

### Maintenance currency observation, 2026-10-04: fetch before source review

Root performed the ADR 0029 maintenance refresh after the bounded deadline correction at source HEAD `402bfd6d1dcd1aee43887fe57032817148a8ed68`, tree `0b3fd806cd8197ccbeb0c01ecbf0c5671a9ffde4`, and before final independent source review. `git fetch origin main` succeeded. The fetched `origin/main` and a subsequent `git ls-remote origin refs/heads/main` both resolved to `d8c6ce8ba495639b8a798e69e58d379d3762b92a`. The additional interval from the prepared donor to that observation contains zero commits, zero changed paths and zero inserted or removed lines. No additional behavior requires disposition at this observation.

This batch stays bounded to d8c6ce8. The observation proves only the maintenance tip read at that point. A pushed candidate does not prove continuing upstream currency. Root must compare maintenance again before later code review or the next product change under ADR 0029. If main advances, classify its exact additional interval, record overlaps and critical fixes, and assign the next normal merge or separately reviewed candidate amendment. Preserve this batch and its exact evidence if upstream continues moving. No newer donor, future maintenance change or current runtime acceptance is implied.

### Independent source acceptance, 2026-10-04

Review: ACCEPT for source preparation only. Reviewed exact HEAD: `103eb29f9b5b8fc718b38f9f051810c64ac57a24`. Reviewed tree: `a571fa02d6c33b7d01d92cbcbf49972478bf237f`. Fresh independent SOL High review receipt: session `01a107c9-97cd-78e3-bb9e-7e5412a98c98`. The reviewer returned no remaining source findings after verifying the bounded corrections, all preserved assertions and migration boundaries, complete changed-input seals, 51 inventory objects, nine baselines, 31 Rust selectors, six support checks, two specification pages and the empty additional maintenance interval. The reviewed c428-relative delta is 58 paths, 7,298 inserted lines and 262 removed lines.

This final receipt changes only this record and its companion. It changes no effective source input or proposed runtime proof. Complete whitespace verification still exits 2 solely for the unchanged XML line 2; source acceptance waives no gate. Compilation, offline COMMAND/ACTUAL acceptance, the actual inventory audit, provenance footprint proof, the 0681 SOURCE amendment and shared integration remain pending with Root. This receipt grants no 0681 CODE authority, runtime qualification or release acceptance.
