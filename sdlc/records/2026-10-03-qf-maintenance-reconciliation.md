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
