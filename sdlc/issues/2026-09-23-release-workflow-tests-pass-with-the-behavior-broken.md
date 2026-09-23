# Release workflow tests pass with the behavior broken

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Symptom

`tests/test_release_workflow_provenance.py` matches workflow text. On a scratch copy of `.github`, these five breaks applied together left all 15 tests passing:

- move the `latest` step ahead of the arm64 smoke
- add `exit 0` at the top of the docs-live loop
- point the smoke at `BIOMCP=biomcp`
- change `needs.docs-live.result == 'success'` to `(... || true)`
- make the upload guard `|| true`

The 1225, 1226, and 1229 records cite these tests as mutation-checked. The mutations tried were narrower than the behavior claimed.

## Fix

- Parse the YAML and assert each job's `needs`, `if`, and step order.
- Move the docs-live and wheel-smoke logic into checked-in scripts. Test them against fake `gh` and `curl` and a fake crashing binary.
- Keep the five breaks above as a mutation list the tests must catch.
