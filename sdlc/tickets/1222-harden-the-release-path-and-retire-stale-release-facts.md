---
flow: build
priority: 4
deps: []
---

# 1222: Keep the release uploads working and the Homebrew formula honest

## Goal

The release workflow stops depending on an archived action and stops being able to write a branch-named Homebrew formula. Published artifacts and their bytes do not change.

## Current Facts

- `.github/workflows/release.yml:98,108` upload assets with `actions/upload-release-asset@v1`, which is archived, and both steps read `github.event.release.upload_url` (`:102`, `:112`). A full `workflow_dispatch` therefore fails at the upload.
- The uploaded assets are the build-matrix artifacts (`release.yml:33-48`) and their `.sha256` sidecars (`:76-96`), produced by the upload step at `:97-116`.
- `homebrew-tap` computes `VERSION="${GITHUB_REF_NAME#v}"` (`release.yml:253`) and commits with `git commit -m "Update biomcp formula for ${GITHUB_REF_NAME}"` (`:281`). On `workflow_dispatch`, `GITHUB_REF_NAME` is the branch, so a manual run can write a formula versioned `main`.
- `container-publish` already resolves the tag once, as `TAG: ${{ github.event.release.tag_name || inputs.tag }}` (`release.yml:295`) with `VERSION="${TAG#v}"` (`:304`). That is the pattern to copy.
- `docs/reference/release-process.md:72-74` states that a dispatch without `container_only` fails at the build job's asset upload. This ticket makes that sentence false.
- Ticket 1220 settled `MAX_PACKAGE_FILES = 1_342` against the correct clean-clone count, so the package-boundary concern in an earlier draft of this ticket needs no work here.

## Design

- Replace both `actions/upload-release-asset@v1` steps with one `gh release upload "$TAG" <files> --clobber` step guarded on `github.event_name == 'release'`, keeping the existing tarball and `.sha256` selection.
- With that guard, a full dispatch proceeds past the upload. PyPI rejects a duplicate version, so name the duplicate-version failure as the dispatch outcome in the runbook rather than leaving the old claim.
- Resolve the tag once in `homebrew-tap` as `github.event.release.tag_name || inputs.tag` and use it for the download URL, `VERSION="${TAG#v}"`, and the commit message.
- Extend `tests/test_release_workflow_provenance.py`: no reference to `actions/upload-release-asset`; the upload step uses `gh release upload` and is release-guarded; `homebrew-tap` derives the tag once and uses it in all three places.
- Update `docs/reference/release-process.md`: the job list at `:11`, the dispatch semantics at `:72-74`, and the witness-gate paragraph at `:82-87` so it points at ticket 1226.

## Acceptance

1. No workflow references `actions/upload-release-asset@v1`; `actionlint` and the new provenance guards pass.
2. `homebrew-tap` uses the resolved tag for the download, the formula version, and the commit message, and a dispatch cannot write a branch-named formula.
3. `docs/reference/release-process.md` describes the new dispatch behavior and no longer claims the asset upload fails.
4. `make lint` passes and CI `canonical-gates` is green on main at the pushed SHA.

## Out of scope

- The docs-live release gate (ticket 1226).
- The retired `release/` package and the stale architecture facts (ticket 1227).
- The cache-expiry flake and any artifact publication change.

## Complexity

- Level 3 (contract 1, state and timing 1, reach 1, proof 1, cost of error 1 = 5)
- Reasons: two trigger paths with an explicit runbook contract; proof is pins and gates, and the upload path is only proven by a real release.

## Review

- Design review: REJECT 2026-09-22 (gpt-5.6-sol, medium) — the original bundle was split into 1222, 1226, 1227, and 1228; this ticket kept the upload and Homebrew work
- Code review: ACCEPT 2026-09-22 (gpt-5.6-sol, medium) — one P1 (dispatch side effects missing from the runbook) and one P2 (upload credential pins) fixed in a172d49a and verified closed
- Verification: actionlint 1.7.12 clean, 11 provenance tests with mutation checks, yellow `make lint` OK at a172d49a; see `sdlc/records/1222-keep-the-release-uploads-working-and-the-homebrew-formula-honest.md`
