# 2074 — Run release checks on every ticket branch

Status: OPEN.

Milestone: 0.9.3

Depends on: 2083
Depends on: 2075

## Outcome

A ticket branch that changes shipped behavior without a changelog bullet, or whose release-time `gh` query would fail, fails CI; a ticket that changes no shipped behavior needs no bullet; and a ticket number in a Rust or Python comment fails lint.

## Evidence

- The 0.9.2 tag needed four rehearsals. Two failed 13 s into the release workflow's version check, each after a 26-minute CI run: one found a missing changelog bullet, the other a `gh` flag only the test stub had (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 9).
- 2057 and 2058 existed only to add changelog bullets for changelog fixes.
- Two of three review rounds on 2052 went to removing ticket numbers from code comments (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 10). Current main has 28 such comments: 1 in Rust (`tests/disable_sources_cli.rs:2`), 13 in Python and 14 in workflow comments.
- On a branch, the green-CI query has no finished run of its own to ask about, so the branch step checks the query runs, not its answer.
- PM decision 2026-10-14: "shipped behavior" means a diff touching `src/`, `templates/`, `skills/`, `docs/`, `crates/`, `Cargo.toml`, `Cargo.lock`, `install.sh` or `pyproject.toml`. The comment lint covers `.rs` and `.py` files only and this ticket removes the existing hits there; workflow comments stay. The branch's green-CI step runs the release's exact `gh run list` command against the merge-base with `origin/main` and fails only when `gh` or `jq` errors. The first-rehearsal measurement belongs to 2082.

- Starts from: `scripts/check-changelog-coverage.py` after 2083 (`--tag` only; finds tickets by landing subject, record and status); the `version-check` job in `.github/workflows/release.yml` (the `gh run list` step); `bin/lint`.
- Keeps: the tag-time version check and `release.yml` unchanged (its steps are hash-pinned in `tests/test_release_workflow_provenance.py`).
- Changes: claims `scripts/check-changelog-coverage.py` `.github/workflows/ci.yml` `tests/test_ci_workflow_contract.py` `tests/test_changelog_coverage_check.py` `bin/lint` `tests/test_lint.py` `tests/disable_sources_cli.rs` `tests/test_alphagenome_proto_generation.py` `tests/test_release_workflow_provenance.py` `tests/test_source_package_boundary.py` `CHANGELOG.md`.
  - Add a `--branch` mode: tickets landed since the last tag, plus the ticket named by `ticket/NNNN-*` in `GITHUB_REF_NAME`; a ticket needs a bullet only when its diff touches shipped behavior as defined above.
  - A `release-checks` job in `ci.yml` runs `--branch`, then the release's `gh run list` command with the real `gh` against the merge-base.
  - `bin/lint` fails on `ticket NNNN` (any case) in a comment in a `.rs` or `.py` file; remove the existing hits in the same ticket. On current main the 13 Python hits sit in `scripts/check-changelog-coverage.py`, `tests/test_changelog_coverage_check.py`, `tests/test_alphagenome_proto_generation.py`, `tests/test_release_workflow_provenance.py` and `tests/test_source_package_boundary.py`, all claimed above.
  - One Internal changelog bullet.
- Proof: a scratch branch with a `src/` change and no bullet fails `release-checks`; one touching only `sdlc/` and `tests/` passes; a planted `// ticket 1234` fails `bin/lint`; main passes after the cleanup. The first two fail on current main.
- Defers: the changelog's shape (2083); the first-rehearsal measurement (2082).
- Order: after 2083 (`scripts/check-changelog-coverage.py`) and after 2075 (`.github/workflows/ci.yml`); before 2076, which also edits `bin/lint`.
