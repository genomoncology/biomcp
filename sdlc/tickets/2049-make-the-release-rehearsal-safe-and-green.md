# 2049 — Make the release rehearsal safe and green

Status: OPEN.

Milestone: 0.9.2

Reviews: revision e0c8d1030, accept

## Outcome

A rehearsal in the scratch repo runs the real `release.yml` from tag to the public-release step and passes, and nothing in it can publish to production PyPI, the production Homebrew tap or the production image. The run ID is recorded on the candidate SHA. The 0.9.2 changelog is true and the coverage check catches a missing bullet.

## Evidence

Filed 2026-10-11 from the independent review of main at `75fabdb20`.

- 2046 closed without its Proof line, "a green rehearsal run ID on the candidate SHA". Rehearsal run 38121524198 at `07d660890` failed in container-publish (`write_package` denied, because the image name was written in as the production name). Run 38124703185 at `75fabdb20` built and smoked all binaries, wheels and both container platforms, then failed in pypi-publish with `invalid-publisher`. publish-release and homebrew-tap never ran.
- The rehearsal now runs the production workflow unchanged. pypi-publish (`release.yml:544-570`) is held off only by PyPI's trusted-publisher setting; if the scratch repo is ever added as a publisher, a rehearsal publishes 0.9.2 to production PyPI. homebrew-tap (`release.yml:576-635`) pushes to the production tap with production URLs (lines 610, 622, 625) and is held off only by a missing `HOMEBREW_TAP_TOKEN` secret.
- `75fabdb20` landed on main with no ticket, no review and no test, and the rehearsal tag was pushed before CI on main finished. The builder's gate `gh run list --branch main --limit 1` can return the docs run, which finishes in about a minute, instead of CI. The diff is correct, but `PINNED_STEPS` in the provenance tests leaves out container-publish, so writing the production name back in passes every test. The hardcoded name came in at `aeea8e142`; the 1 October rehearsal hid it by editing its own copy of the workflow.
- `CHANGELOG.md:30` says a shared product name "can still return one drug's identifiers with another's targets, which ticket 2039 tracks". 2039 fixed that and has no bullet. `scripts/check-changelog-coverage.py:198` counts any number anywhere in a bullet, so that sentence counts as coverage of 2039. `tests/test_docs_changelog_refresh.py:20-22,213` pins the stale sentence. The 2045, 2046 and 2048 bullets (`CHANGELOG.md:4-6`) sit above every subheading.
- The scratch repo's "Publish documentation" runs fail because `docs-edge.yml:28` fetches a `gh-pages` branch the scratch repo lacks. docs-live passed only because biomcp.org already serves the same commit. The rehearsal left a draft v0.9.2 release and an image package in the scratch repo.
- `Dockerfile:20` writes the production repo into `org.opencontainers.image.source`, so the rehearsal image claims the production source.

- Starts from: main `75fabdb20` and ticket 2046.
- Keeps: 2046's fixes (no duplicate keys, `-e TAG`, the duplicate-key and docker-env tests, both "Land" subject forms in the gate); the derived image name; the production release behavior when the workflow runs in genomoncology/biomcp.
- Changes: run the upload step of pypi-publish and the push step of homebrew-tap only when `github.repository == 'genomoncology/biomcp'`, with `twine check` and a dry formula render otherwise, so publish-release runs in the rehearsal; add a provenance test that every `ghcr.io/` reference and the publish guards come from the repository name; skip the docs-edge job outside the production repo; pass the image source as a build argument; wait for the CI workflow on the exact SHA (`gh run list --workflow ci.yml --commit SHA`) before tagging; add a 2039 bullet, rewrite the 2031 sentence and its pin, move the three loose bullets under Internal; make the coverage check count only the closing `(NNNN, …)` group; delete the scratch repo's draft release and package before each rehearsal.
- Proof: a provenance test that fails when the production image name or an unguarded upload is put back; a changelog test that fails when a closed ticket's number appears only in prose; a green rehearsal run on the candidate SHA in genomoncology/biomcp-release-rehearsal, run ID recorded here, with PyPI still showing 0.9.1 afterwards.
- Defers: nothing.

## Build status (2026-10-11)

- Landed through merge e0c8d1030 (branch tickets/2049-rehearsal-safety
  at 4b0642209 over main). CI green on the exact merge (run 38141311569,
  the ci.yml workflow on commit e0c8d1030).
- Green rehearsal: run 38143192187 in genomoncology/biomcp-release-rehearsal
  on the candidate tag v0.9.2 = e0c8d1030, every job success through
  publish-release: all five platform builds, all five wheel smokes
  (each printed exactly `biomcp 0.9.2` and passed the exact comparison),
  both container platforms, the derived image name pushed to the scratch
  namespace, the guarded pypi-publish ran the credential-free check
  (all five 0.9.2 wheels PASSED twine), homebrew checked its inputs
  without pushing, and publish-release completed. PyPI still serves
  0.9.1 as latest after the run.
- The review chain: the first pass at f62a2160a found two blockers
  (the guard test missing the tagged production image name; the
  coverage gate losing period-trailed closing groups); both were fixed
  at 4b0642209 and re-verified by the same reviewer with the mutation
  rerun and the v0.9.1 gate passing; the final verdict is accept.
- Stated rehearsal differences, all recorded in the workflow comments:
  the twine check for the upload, the dry input check for the tap, the
  docs-live stub (biomcp.org serves the production repository), and the
  scratch gh-pages seeded for the Pages workflow.
