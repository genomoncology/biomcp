# 2161 — Rehearse and qualify releases without push CI

Status: OPEN.

Milestone: 0.9.3

Reviews: revision d6adf66, accept

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
