# Release workflow tests describe a retired workflow

Filed 2026-09-18 while clearing the gate for ticket 1202.

## Symptom

`make test` fails with 23 Python failures. Every one of them is in the release
workflow family:

- `tests/test_release_stage_workflow.py`, 17 failures
- `tests/test_release_workflow_provenance.py`, 2 failures
- `tests/test_upstream_planning_analysis_docs.py`, 2 failures
- `tests/test_public_installer_checksum.py`, 1 failure
- `tests/test_routine_cargo_feature_contract.py`, 1 failure

Two executable spec blocks fail for the same reason, found once the spec
suite was unmasked:

- `spec/surface/docker-image.md:30`, `Stage And Promotion Stay Separate`
- `spec/surface/homebrew.md:30`, `Public Tap Moves Only After Verification`

Both match `.github/workflows/release.yml` against staged-release markers such as
`mode:`, `publish-versioned:`, and `advance-mutable-pointers:`, none of which the
current workflow contains.

The tests assert a staged release design against `.github/workflows/release.yml`:

```
assert set(dispatch) == {"mode", "source_sha", "stage_run_id", "windows_desktop_smoke", ...}
assert text.count("ref: ${{ inputs.source_sha }}") >= 3
```

The current `release.yml` has none of `mode`, `source_sha`, or `stage_run_id`.
It is triggered by a published release or by a `tag` input, builds five
platform assets, uploads them, publishes wheels, and updates the Homebrew tap.

## Cause

Commit `7a6b6cf0` on 2026-09-16, `Restore the proven release workflow for 0.9.0
publication`, replaced the staged workflow: 373 insertions, 956 deletions. The
tests were written in August under tickets 0988 and 0991 for the design that
commit retired, and they were not removed with it.

The workflow is the intended one. The tests are stale. Confirmed pre-existing at
`51811699`, before any cell-line work.

## Why it was not obvious

These tests only run in the Python half of `make test`, and the quality ratchet
that runs beside them was itself failing for an unrelated reason, so the whole
area read as uniformly red. Two of the failures in the original count of 25 were
not real: they shelled out to the ratchet and passed as soon as it was green.

## Fix direction

Decide which artifact is authoritative, then make the other match. Ian retired
the staged workflow deliberately, so the likely answer is that the tests go.

1. Delete the tests that describe the retired design, or rewrite them against
   the workflow that exists. `test_release_stage_workflow.py` is entirely about
   `stage` and `promote` modes and has no meaning without them. The two spec
   blocks go the same way as whatever is decided for the tests, since they
   assert the same markers.
2. Check the four failures outside that file individually. They may be asserting
   something the current workflow could reasonably satisfy, in which case the
   workflow is what should change.
3. Retire the planning prose that still describes the staged flow, so the next
   reader does not reconstruct it.

Until this is settled `make test` cannot pass for anyone, which means the suite
gives no signal about whether a change broke something.

## Not in scope here

The separate masking problem is filed in
`2026-09-18-one-failing-spec-page-hides-the-whole-spec-suite.md`. The two are
independent, but they share a shape: a check that has been failing long enough
that its red is read as normal.

## Resolved

Ian ruled the restored `.github/workflows/release.yml` authoritative. The
artifacts that described the staged `stage`/`promote` design are retired.

Deleted:

- `tests/test_release_stage_workflow.py`, 17 tests, entirely about `stage` and
  `promote` modes.
- `spec/surface/docker-image.md`, `Stage And Promotion Stay Separate`.
- `spec/surface/homebrew.md`, `Public Tap Moves Only After Verification`.

The six remaining failures were decided one at a time.

1. `test_release_workflow_provenance.py::test_write_permissions_exist_only_on_the_two_protected_pointer_paths`
   asserts the job names `publish-versioned` and `advance-mutable-pointers` and
   the `biomcp-release-promotion` environment, none of which exist outside the
   staged design, so the test is removed.
2. `test_release_workflow_provenance.py::test_stage_is_read_only_and_latest_waits_for_public_reconciliation`
   selects jobs by `if: inputs.mode == 'stage'` and the current workflow has no
   `mode` input at all, so the test is removed.
   `test_no_other_workflow_exposes_release_publication` still holds and stays.
3. `test_upstream_planning_analysis_docs.py::test_technical_and_ux_docs_match_current_cli_and_workflow_contracts`
   asserted `contents: read`, `promotion-preflight:`, `reconcile-public-release:`
   and `advance-mutable-pointers:`; the current workflow declares
   `contents: write` at the top level and none of those jobs, so those four
   markers are replaced with `types: [published]`, `environment: pypi` and
   `homebrew-tap:`, which the workflow does contain.
4. `test_upstream_planning_analysis_docs.py::test_pull_request_contracts_remain_separate_from_protected_release`
   is a real separation contract and its CI half already passes, so only the
   three staged markers are swapped for the protected `pypi` environment and a
   check that no `make lint`/`make test` gate leaks into the release workflow.
5. `test_public_installer_checksum.py::test_ci_and_release_gate_installer_identity_before_docs_or_release`
   required `cmp --silent install.sh docs/install.sh` to precede
   `release/candidate.py init` in the release workflow; that step does not
   exist, and `.github/workflows/ci.yml` already runs the same check on every
   pull request and push to main, so the test keeps only the CI half and is
   renamed `test_ci_proves_installer_identity_before_the_version_sync_check`.
6. `test_routine_cargo_feature_contract.py::test_release_staging_runs_and_records_the_named_all_feature_proof`
   required `make full-feature-check` and a `for gate in lint test
   full-feature-check spec` loop inside the release workflow; the restored
   workflow runs no gates, and the all-feature proof is already asserted in
   `test_release_gate_runs_a_named_all_feature_check` (Makefile `release-gate`)
   and `test_ci_and_developer_docs_name_small_and_full_feature_lanes`
   (`ci.yml`), so the test is removed rather than duplicated.

Stale prose: the `Release Pipeline` section of
`architecture/technical/overview.md` described the `stage`/`promote` modes and
the sealed-candidate promotion. It now describes the workflow that exists and
says the `release/` Python package is not wired into it.
`tests/test_docs_changelog_refresh.py::test_release_overview_describes_committed_metadata_and_protected_promotion`
was updated to match.

`tests/test_source_package_boundary.py` carries an exact packaged-file count.
Deleting one test file moved `MAX_PACKAGE_FILES` from 1,324 to 1,323.

Verified with `uv run --extra dev pytest tests/ -q`:

- Before: `30 failed, 945 passed, 1 skipped`. 23 of those were the release
  workflow family. Six are pre-existing local-environment failures unrelated to
  this issue, and one was the spec runner contract updated under the masking
  issue.
- After: `6 failed, 948 passed, 1 skipped`. The 23 release workflow failures are
  gone. The remaining six are the same pre-existing local-environment failures:
  two article fixture pages, the offline corpus benchmark, the wheel install,
  the zero-coupled source package build, and the `TMPDIR`-outside-the-worktree
  contract. They fail identically at `40ef3c58`.
