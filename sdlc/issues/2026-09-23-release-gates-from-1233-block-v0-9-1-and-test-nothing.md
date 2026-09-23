# Release gates from ticket 1233 block v0.9.1 and test nothing

Filed 2026-09-23 from the independent review of ticket 1233 (merge 1cf8e278, follow-up 086f40e5). Blocks 0.9.1. The reviewer ran the version step and the coverage script locally with a stub `gh`, and ran workflow mutations on a scratch copy.

## The changelog gate rejects the release's own changelog

`scripts/check-changelog-coverage.py:66` reads only `## Unreleased`. The regex also needs a following `## ` heading. The v0.9.0 tag carried `## 0.9.0 — 2026-09-16` and no Unreleased heading. Renaming the heading the same way for 0.9.1 makes the script exit 1 with "has no Unreleased section". An Unreleased section at the end of the file fails the same way.

Fix: read the `## <tag version>` section and fall back to Unreleased. Match up to the next heading or end of file. Fail with a clear message when neither exists.

## Ticket discovery is wrong and the check is easy to satisfy

The script matches any `1[0-9]{3}` in any commit message (`:11`). Against the real v0.9.0..HEAD history it reports 25 missing tickets, including 1204-1216 and 1220-1233. It counts 1086, which one test commit only mentions. It never sees a ticket numbered 2000 or higher. A bare list of the numbers passes.

Fix: take tickets only from `Merge ... tickets/NNNN-` commits. Require each on a bullet line with descriptive text. Allow an explicit internal-only marker for tickets with no user-visible change. Use `git log prev..tag` on a `fetch-depth: 0` checkout. The compare API at `release.yml:50` caps at 250 commits and silently drops the rest.

## The tests pass with the gates removed

On a scratch copy, the provenance tests stayed 23/23 green after each of these edits: mismatch `exit 1` changed to `exit 0`, `|| true` appended to the coverage call, `continue-on-error: true` on `version-check`, and `if: false` on the version step. The version logic is inline shell (`release.yml:39-47`) and nothing executes it.

Fix: move the version check into `scripts/` and test its behavior: match, mismatch, missing `v`, pre-release. Add workflow assertions that each gate step has no `continue-on-error`, no `if:` escape, and no `|| true`. Prove each assertion by mutating the workflow in the test.

## The version check is narrower than claimed

It compares Cargo.toml and pyproject.toml only. `server.json` still says 0.9.0. Cargo.lock, uv.lock, manifest.json, and CITATION.cff are unchecked. `scripts/check-version-sync.sh` already checks all of them, and the release never calls it. A tag without `v` passes, yet the Homebrew formula builds URLs from `v${VERSION}` (`:392`). An rc tree with Cargo `0.9.1-rc.1` and pyproject `0.9.1rc1` can never pass.

Fix: call `check-version-sync.sh` from the release. Require `^v` on the tag. Map Cargo pre-release forms to PEP 440 before comparing.

## The GitHub release goes public before any check runs

The trigger is `release: published` (`:4-5`). A failed check leaves a public release with no assets. The container job depends on `version-check` only through its `docs-live` success condition (`:415`).

Fix: have the workflow create the release as a draft and publish it last, or trigger on the tag push. Add `version-check` to the container job's `needs` and pin every publish job's chain in a test.

## The wheel smoke covers one platform and passes panics

It installs the Linux x86_64 wheel only. macOS and Windows wheels reach PyPI unsmoked. `run_smoke` (`release.yml:214-231`) fails on 134, overflow text, or exit 128 and above. A Rust panic exits 101 and passes. Ticket 1230's record says the opposite.

Fix: run the smoke as a matrix over every built wheel. Require exit 0 from the deep-path commands and the exact expected code from the not-found command.

## Minor

- The coverage step calls `python` (`:53`). Use `python3` or add `setup-python`.
- Workflow-level permissions are write-all (`:18-21`). Already filed in `2026-09-23-release-workflow-gating-and-permissions.md`. Fix it in the same pass.
- Tag handling is safe: every `run:` step reads the tag through `env`.
