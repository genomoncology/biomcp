# 2125 — Close the release-gate holes the 0.9.3 review found

Status: OPEN.

Milestone: 0.9.3

## Outcome

A release tag needs a green `main` push run of ci.yml on its commit, the changelog coverage check charges each landing only with its own files, the packaged-crate cold compile runs before every release, and no duplicate changelog bullet can land.

## Evidence

Found by the 0.9.3 adversarial review on 2026-10-16.

- The release step counts any successful ci.yml run on the tagged commit. PM ticket branches fast-forward to main, so one commit can carry a branch run that skips Windows, all-features, stress and release-panic. 393737eae has a failed main run and a passing branch run; a tag there would have released. 2075's claim that every tag needs those jobs is false.
- `scripts/check-changelog-coverage.py` attributes files with `git diff-tree -r -m --first-parent <merge>`; on a merge `-m` adds the diff against the second parent, so a stale branch's landing is charged with everything main gained since it forked. The 2116 landing was charged with 59 files and turned main red.
- 2071 moved the packaged-crate cold compile behind `-m release_gate` (`Makefile:68,87`); nothing in CI, `release.yml` or `docs/reference/release-process.md` runs it.
- The 2079 bullet appears twice in the 0.9.3 section, with a stray blank line, and 2083's shape test passed.
- 2072 uses the std file-lock API (Rust 1.89 or later) and `Cargo.toml` declares no `rust-version`.

- Starts from: `.github/workflows/release.yml`, `scripts/check-changelog-coverage.py`, `Makefile`, `CHANGELOG.md`, `tests/test_docs_changelog_refresh.py`, `Cargo.toml`.
- Decisions:
  - The release check counts only ci.yml runs with event `push` and branch `main`.
  - The coverage check charges a merge with `git diff --name-only <merge>^1 <merge>`.
  - The release workflow runs the release-gate marker set before it builds archives.
  - The changelog shape test fails on a repeated bullet or a blank line inside a list; this ticket deletes the duplicate 2079 bullet.
  - `Cargo.toml` sets `rust-version` to the pinned toolchain's minimum the code needs.
- Changes: claims `.github/workflows/release.yml` `scripts/check-changelog-coverage.py` `tests/test_changelog_coverage_check.py` `tests/test_release_workflow_provenance.py` `tests/test_docs_changelog_refresh.py` `Makefile` `Cargo.toml` `CHANGELOG.md` `docs/reference/release-process.md`.
- Proof, each failing on current main: the release check refuses a commit whose only green run is a branch run; the coverage check charges a stale-branch landing only with its own files; the shape test refuses a duplicated bullet. A release rehearsal in the release-rehearsal repository shows the release-gate step running.
- Order: before release.
