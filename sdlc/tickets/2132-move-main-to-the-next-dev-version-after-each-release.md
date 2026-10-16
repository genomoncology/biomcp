# 2132 — Move main to the next dev version after each release

Status: OPEN.

Milestone: 0.9.3

## Outcome

A build from main between releases reports the coming version as a dev build, such as `0.9.4-dev+g1a2b3c4d`, never the last released number.

## Evidence

- Main's `Cargo.toml` still says `0.9.2` while 0.9.3 work lands, so a build from main reports `0.9.2+g<sha>` (`tools/with-build-identity`). People and agents read that as 0.9.2 and cannot tell a current build from an old install.

- Starts from: `Cargo.toml`, `tools/with-build-identity`, `scripts/check-version-sync.sh`, `docs/reference/release-process.md`, `sdlc/release-checklist.md`.
- Decisions:
  - Right after a release tag, main's version becomes the next patch version with a `-dev` suffix; the release commit drops the suffix. Main moves to `0.9.3-dev` now.
  - Keep every version-sync check passing for both shapes; the release workflow refuses a tag whose version carries `-dev`.
- Changes: claims `Cargo.toml` `Cargo.lock` `pyproject.toml` and the other version-synced files `scripts/check-version-sync.sh` `tools/with-build-identity` `.github/workflows/release.yml` `docs/reference/release-process.md` `sdlc/release-checklist.md` `CHANGELOG.md` and their tests.
- Proof: a build from main prints `0.9.3-dev+g<sha>`; the version-sync check passes; a release rehearsal in the release-rehearsal repository refuses a `-dev` tag and passes after the release commit drops it.
- Order: safe alongside anything; touches `release.yml` beside 2125, so rebase after it.
