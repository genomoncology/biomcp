# 2060 — Make the release CI check use flags the gh CLI has

Status: OPEN.

Milestone: 0.9.2

## Outcome

The release workflow's green-CI check runs on the gh version GitHub runners ship, so a tag whose CI passed releases.

## Evidence

Filed 2026-10-13 from rehearsal round 3 (run 38337603835): version-check's new CI-success step failed with "unknown flag: --conclusion" — `gh run list` has no `--conclusion` flag on the runner's gh or on gh 2.45.0 locally. The step was tested only against a stub gh that accepts any flag.

- Starts from: main `90221a07f`.
- Changes: the step filters with `--json conclusion --jq '[.[] | select(.conclusion == "success")] | length'` instead of `--status completed --conclusion success`; the pinned step hashes and any stub-gh argument assertions in tests/test_release_workflow_provenance.py move with it; the guard test gains a check that the command's gh flags parse against a real gh binary (flag errors happen before any network call), so an unsupported flag fails the test that once passed a stub.
- Proof: the targeted workflow tests pass; the flag-surface check fails when `--conclusion` returns; rehearsal round 4's version-check passes the CI-success step.
- Defers: nothing.
