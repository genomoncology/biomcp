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

They assert a staged release design against `.github/workflows/release.yml`:

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
   `stage` and `promote` modes and has no meaning without them.
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
