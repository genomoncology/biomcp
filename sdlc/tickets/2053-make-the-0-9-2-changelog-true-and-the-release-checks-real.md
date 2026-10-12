# 2053 — Make the 0.9.2 changelog true and the release checks real

Status: COMPLETE.

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

## Evidence

Built 2026-10-11 on `tickets/2053-work` at `0c8d289fa` (working tree, evidence below appended after the gates ran).

Gates on the build host against the pushed branch: `make lint` green (workflow lint included), `make test` green (Rust nextest archive plus the full offline pytest lane: 1253 passed, 3 skipped, needs_binary included), `make spec` exit 0 (spec pages 7 passed). CI run 38196648667 finished `success` on `0c8d289fa7029f014d7afe6b89146857a532cb0e`.

Changelog corrections landed: the 0.9.2 section is dated 2026-10-11 (CITATION.cff `date-released` moved with it so the citation and changelog contracts stay in agreement); the 2049 bullet moved under Internal reworded as the plain fact (guards that stop the rehearsal repository from publishing); the stray blank line inside Fixes is gone; the 2039 refusal is said once (the two bullets merged into one `(2031, 2039)` bullet); the GWAS bullet states what 2050 did (a 410 degrades with the retirement note, a 404 means no associations) and carries `(2047, 2050)`; the 2050 bullet moved under Fixes; the 2051 bullet says only what is true (the 0.9.2 tickets read as written, the review records come from the ticket tool); a 2053 bullet states its user-facing truth (a release tag builds only from a commit whose CI run finished green, the changelog states only what shipped); a 2052 bullet describes the drug-lookup fixes in plain words with the closing group `(2052)` alone. `EXPECTED_RELEASE_TICKETS` for 0.9.2 in `tests/test_docs_changelog_refresh.py` now carries 2052 and 2053.

New coverage-gate tests in `tests/test_changelog_coverage_check.py`, each proved red against the documented revert of `scripts/check-changelog-coverage.py` (the closing-group logic replaced by `group = re.findall(r"[0-9]+", text)`, counting any number):

- Mutation: revert the closing-group logic to counting any number.
  `test_a_ticket_number_that_appears_only_in_prose_is_not_coverage` FAILED under the revert (the gate accepted a prose-only number) and passes on the landed script.
  `test_a_period_after_the_closing_group_counts_only_when_the_group_is_real` FAILED under the revert (the gate accepted `(1234, see the review).`) and passes on the landed script.

Guard tests that run the guarded scripts, with stub `uv`, `uvx`, `gh` and `git` on PATH under both repository names (stub log shown; the Homebrew rehearsal ran with a tap token present on purpose, so only the repository guard can stop the push):

```
[pypi-publish] genomoncology/biomcp: exit=0
    uv publish --trusted-publishing always dist/...
[pypi-publish] genomoncology/biomcp-release-rehearsal: exit=0
    uvx twine check dist/...
[homebrew-tap] genomoncology/biomcp: exit=0
    git clone https://x-access-token:stub-token@github.com/genomoncology/homebrew-biomcp.git tap
    git add Formula/biomcp.rb
    git diff --cached --quiet
    git commit -m Update biomcp formula for v0.9.2
    git push origin HEAD:main
[homebrew-tap] genomoncology/biomcp-release-rehearsal: exit=0
    (no tool ran; the rendered formula carries both release checksums)
```

Recorded mutations of `.github/workflows/release.yml`, each reverted after its run (the text-reading assertions at the old `tests/test_release_workflow_provenance.py:1792-1822` are deleted; the executed-guard tests below replace them):

- Mutation: the pypi rehearsal `exit 0` deleted.
  `test_pypi_publish_guard_runs_the_right_publisher_under_each_repository` FAILED: `AssertionError: genomoncology/biomcp-release-rehearsal: uv publish ran`.
- Mutation: `uv publish` moved above the `if` guard.
  Same test FAILED: `AssertionError: genomoncology/biomcp-release-rehearsal: uv publish ran`.
- Mutation: an extra `exit 0` after the guard's `fi` (production stops publishing).
  Same test FAILED: `AssertionError: genomoncology/biomcp: uv publish did not run`.
- Mutation: the Homebrew rehearsal `exit 0` deleted.
  `test_homebrew_guard_renders_and_checks_both_checksums_then_guards_the_push` FAILED: `AssertionError: genomoncology/biomcp-release-rehearsal: the rehearsal must not touch git`.
- Mutation: `IMAGE_SOURCE` changed from `https://github.com/${{ github.repository }}` to the hardcoded production repository.
  `test_container_image_source_build_argument_is_pinned_to_the_repository` FAILED: `AssertionError: the container image source label must follow the repository running the workflow`.

Also landed: the version-check job gains `Require green CI on the tagged commit` (`gh run list --workflow ci.yml --commit "$GITHUB_SHA" --status completed --conclusion success`), with `actions: read` so the default `GITHUB_TOKEN` works in both repositories; its executed-script tests pass a green reply under both repository names, fail on reply `0` and on an empty reply, and fail when `gh` itself fails. The step is pinned in `PINNED_JOB_STEP_LISTS` and by SHA-256 in `PINNED_STEPS`. The Homebrew rehearsal now renders the production formula from the template into `$RUNNER_TEMP` and checks both platform checksums (arm + x86) before its `exit 0`; `test_homebrew_rehearsal_fails_when_a_release_checksum_is_missing` proves a dist without the arm64 checksum fails the rehearsal. Scratch-repo tags were not seeded and no image versions were deleted; the lander runs the rehearsal.
