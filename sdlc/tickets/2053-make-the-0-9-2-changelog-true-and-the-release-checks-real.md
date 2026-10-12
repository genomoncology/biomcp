# 2053 — Make the 0.9.2 changelog true and the release checks real

Status: OPEN.

Milestone: 0.9.2

Reviews: revision 8fec23f, accept

## Outcome

Every 0.9.2 changelog sentence is true and sits in the right group. A tag releases only from a commit whose CI passed. The rehearsal tests what production will do, and a test fails if a production upload guard stops guarding.

## Evidence

Filed 2026-10-11 from the independent review of main at `b039ac3e7` (candidate `91dbbeaec`).

- `CHANGELOG.md:39` says the GWAS section degrades when the endpoint "answers 410 (or 404)". 2050 made a 404 mean no associations (`src/sources/gwas.rs:100-117`).
- `CHANGELOG.md:6` says "A rehearsal can no longer publish anywhere but the production repository", which reads as the opposite of what 2049 did. It is release plumbing and sits alone under Fixes.
- `CHANGELOG.md:54` files 2050's user-facing drug, disease and GWAS fixes under Internal. `CHANGELOG.md:55` describes the ticket tool's inbox, which is not about this product, and claims records work that ticket 2054 shows is not done. `CHANGELOG.md:7` is a blank line inside the Fixes list. `CHANGELOG.md:28` and `:29` both describe the 2039 refusal. `CHANGELOG.md:3` is dated 2026-10-10.
- 2049's Proof promised a changelog test that fails when a ticket number appears only in prose. No test does this: with `scripts/check-changelog-coverage.py:198-214` reverted to counting any number, every test still passes.
- `tests/test_release_workflow_provenance.py:1792-1822` checks the guard text. It still passes if the guard's `exit 0` (`release.yml:578`, `:618`) is deleted or `uv publish` moves above the `if`, which would upload from the rehearsal repo, and if an extra `exit 0` stops production from publishing. No test pins the `IMAGE_SOURCE` build argument (`release.yml:744`). The PyPI trusted-publisher setting and the missing tap token still stop a real upload today.
- The Homebrew rehearsal (`release.yml:614-618`) runs `test -n "$VERSION"` and prints that the formula reads the release checksums, but reads nothing. 2049 asked for a dry formula render.
- Nothing in the repo waits for CI on the exact commit before a release. `version-check` (`release.yml:35-51`) checks versions and the changelog only, so a tag on a commit with failing CI would release. The builder's last landing waited with `gh run list --branch main --limit 1`, which can pick the docs run.
- In rehearsal run 38174801040 the changelog gate printed "no previous release; nothing to cover", because the scratch repo has no v0.9.1 tag. The rehearsal did not exercise the gate.
- The scratch image `ghcr.io/genomoncology/biomcp-release-rehearsal` still serves public `0.9.2` and `latest` tags.
- Five commits reached main with failing CI: `9c45d0123`, `cbe3b109e`, `07fea376e` (the 2049 landing), `4d5ce8bcc` and `60bd97943`. Each pushed a landed ticket's record before its verdict line existed, so the records contract test failed until the next push.

- Starts from: ticket 2049 and its landing `e0c8d1030`.
- Keeps: the repository guard on the PyPI upload and Homebrew push; the derived image name and source; the rehearsal through publish-release in the scratch repo only.
- Changes: correct the changelog lines above and move 2049's and 2051's bullets under Internal as plain product facts; add direct tests for prose-only numbers and a period after the closing group; test the guarded scripts by running them with stub `uv`, `uvx` and `git` under both repository names and asserting which ran; pin `IMAGE_SOURCE`; render the formula and check both checksums before the rehearsal exits; add a `version-check` step that requires ci.yml success on `$GITHUB_SHA`; seed a v0.9.1 tag in the scratch repo so the changelog gate runs; delete the scratch image versions after each rehearsal; record a ticket's verdict before or in the same push as its landed record.
- Proof: each new test fails when its guard is removed or inverted; a rehearsal run whose changelog gate covers the 0.9.2 tickets and whose Homebrew job renders a formula; PyPI still at 0.9.1; ci.yml green on the exact commit that gets tagged.
- Defers: nothing.
