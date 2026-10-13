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

Build 2026-10-13 on ticket/2060-gh-flags: the CI gate now runs `gh run list --workflow ci.yml --commit "$GITHUB_SHA" --limit 100 --json conclusion --jq '[.[] | select(.conclusion == "success")] | length'` and refuses on an empty count, on zero, and on anything that is not a plain number, so a failed gh or jq refuses too; the step's SHA-256 pin moved with it. `test_the_ci_gate_flags_parse_on_the_real_gh_cli` extracts the step's exact invocation and runs it against the real gh on PATH (a deliberately invalid token; flags parse before any network call), failing on "unknown flag" and on gh's usage text. Red proof: restoring the pre-2060 invocation (`--status completed --conclusion success --json databaseId --jq length`) makes that test fail with gh 2.45.0's `unknown flag: --conclusion`, exactly the rehearsal failure; the fixed branch passes. Targeted on the build host: the full workflow-provenance module plus the changelog refresh and coverage modules (185 tests), then make lint and make spec, all green. The changelog gains the Internal (2060) bullet and the expected-ticket set gains 2060, so the release gate at the tag finds this ticket described. Status stays OPEN for the lander.
