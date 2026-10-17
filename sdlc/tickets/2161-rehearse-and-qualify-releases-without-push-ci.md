# 2161 — Rehearse and qualify releases without push CI

Status: OPEN.

Milestone: 0.9.3

Reviews: revision d6adf66, accept

Reviews: revision f5b2e64, accept

Reviews: revision 1bb445b, accept

## Outcome

A release qualifies its exact revision directly, and the full release workflow can rehearse an untagged candidate without publishing before Ian tags it.

Coordinator qualification finding: stable metadata proof job `1792244785321-3844` exercised this branch's revision-based changelog checker against the prepared candidate and rejected missing historical references 2041, 2054, 2063, 2096, 2102, 2105 and 2148. Before changing qualification or descriptions, classify each against actual first-parent release landings and ticket milestones. Preserve valid missing-entry detection and avoid importing unrelated merged history into the release. Correct only the established qualification defect or factual missing descriptions, with a failing entry-point regression for any checker change.

Depends on: 2156
Depends on: 2159

- Starts from: `.github/workflows/release.yml`, the manually runnable CI/documentation workflows and their existing provenance and behavior contracts. Current release validation requires a main-push CI run, and ordinary dispatch only checks an existing tag; neither serves the approved process.
- Keeps: five platform archives and wheels, checksums, existing package and container platform smokes, version/changelog checks, publication ordering, least permissions and existing container-only dispatch.
- Changes: claims `release/homebrew_formula.py` `tests/test_release_versions_check.py` `.github/workflows/release.yml` `.github/workflows/ci.yml` `.github/workflows/docs-edge.yml` `scripts/check-release-versions.py` `scripts/check-changelog-coverage.py` `tests/test_release_workflow_provenance.py` `tests/test_ci_workflow_contract.py` `tests/test_docs_publication_contract.py` `tests/test_changelog_coverage_check.py` `docs/reference/release-process.md` `sdlc/release-checklist.md` `CHANGELOG.md`. Report exact additional existing helper/test paths before changing them.
- Proof: tests through the workflow/helper entry points fail on the old manual/tag behavior; prove manual rehearsal runs every build and validation lane, requires no tag and cannot reach a publisher. Verify successful/failed qualification and docs admission, wrong candidate/version, tag and container-only paths. Run targeted checks through the build job. The final measurement/release proof includes one complete successful manual rehearsal of the final release commit before any tag.
- Defers: actual release tagging and publication to Ian.

PM decision: 2026-10-17. Make CI reusable from the release workflow and remove dependence on preceding main-push CI. Real tag releases publish their tagged documentation before verifying its live revision; retain protection against older releases replacing newer docs. Add a manual rehearsal mode accepting a full candidate SHA and intended stable version without creating any tag. Preserve existing dispatch inputs and container-only behavior; reject conflicting modes. Rehearsal builds and validates production artifacts through the same build paths, keeps internal run artifacts, validates generated docs and their revision marker, builds/smokes container platforms without registry pushes, and validates publication payloads without release creation, package upload, website deployment or Homebrew changes. Publication credentials and write permissions remain confined to actual publishers. Record real-publication-only steps as unexercised. Pin the workflow definition and product source to the candidate being rehearsed; changed release code/workflow requires another full rehearsal.

Integration review: combine accepted sources 2073 `4428b4e671`, 2095 `6fc832bee`, 2139 `ac7f5a4f3` and 2170 `fe548b1e2` with its proof record `cf128d382`, preserving their reviews. Resolve the CI merge by retaining reusable exact-revision qualification and optimized full-feature checks alongside Windows-only dispatch, heavy-job skips and Bash failure propagation. Keep the routine warning reporter, release-candidate long suite, development-profile main checks and required audit stage. Changelog entries retain their existing headings and ticket suffixes.

Integration proof: build job `1792253732842-77560` passed 289 focused workflow, release/version/changelog, routine parallelism, slow-warning and shared-helper contracts; 53 cases requiring a built binary or outside this selection were deselected. The patch against published `1038b950f` included integrated source and contract changes but omitted only the already accepted full-text recording payloads to fit dispatch arguments; their earlier product evidence remains applicable. Build job `1792253779215-81759` passed Ruff on the changed CI contract and actionlint on CI, release and documentation workflows. No Rust build, full gate or hosted workflow was run for this integration. The coordinator still requires the complete pushed candidate landing gate and actual-main check; the final complete nonpublishing release rehearsal remains required.

Final batch preparation includes accepted measurement-summary source a970a2e26 and main 38610b465, carrying the reviewed startup correction and request diagnostics. Only changelog and record merges followed the integration review; no additional implementation conflict occurred. Preserve the individual accepted sources for 2073, 2095, 2139, 2169 and 2170, with 2161 as the carrier. Reuse the unchanged measurement parser's seven-case and complete coverage evidence. Actual main must pass its full check before this batch lands; the complete combined candidate still receives one landing gate and a full check on actual main afterward. The final hosted nonpublishing rehearsal remains after final measurements.
