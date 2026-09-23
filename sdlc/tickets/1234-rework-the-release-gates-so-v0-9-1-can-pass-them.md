# Rework the release gates so v0.9.1 can pass them

Filed from `sdlc/issues/2026-09-23-release-gates-from-1233-block-v0-9-1-and-test-nothing.md`. Blocks 0.9.1. The reviewer proved the current gates would block the release itself and that the tests stay green with the gates removed.

## Design

1. **Changelog section**: `scripts/check-changelog-coverage.py` reads the
   `## <tag version>` section and falls back to `## Unreleased`, matching
   up to the next heading or end of file, failing with a clear message
   when neither exists. A release that renames Unreleased to the version
   heading must pass.
2. **Ticket discovery**: tickets come only from `Merge ... tickets/NNNN-`
   commit subjects, collected with `git log <previous>..<tag>` on a
   `fetch-depth: 0` checkout (the compare API caps at 250 commits).
   Ticket numbers of any length are recognized. Each ticket needs a
   described bullet in the section; a bare number list fails. Tickets with
   no user-visible change need an explicit internal-only marker bullet.
3. **Version check as a script**: move the comparison into
   `scripts/check-release-versions.py` with behavior tests for match,
   mismatch, a tag missing the `v` prefix, and a pre-release tree
   (Cargo `0.9.1-rc.1` maps to PEP 440 `0.9.1rc1`). The workflow calls the
   script. Workflow assertions prove every gate step has no
   `continue-on-error`, no `if:` escape, and no `|| true`, and each
   assertion is proven by mutating the workflow inside the test.
4. **Version width**: the release runs `scripts/check-version-sync.sh`,
   which already covers `server.json`, `Cargo.lock`, `uv.lock`,
   `manifest.json`, and `CITATION.cff`. The tag must match `^v`.
5. **Draft-last publishing**: the release event no longer publishes before
   the checks. The workflow creates the GitHub release as a draft and
   publishes it as the final step, or triggers on the tag push with the
   same effect. `container-publish` needs `version-check` directly, and a
   test pins every publish job's full `needs` chain.
6. **Wheel smoke matrix**: the smoke runs on every built wheel (Linux
   x86_64, macOS arm64, macOS x86_64, Windows). Deep-path commands must
   exit 0; the not-found adverse-event command must exit with its exact
   expected code and text. A panic (exit 101) fails. `1230`'s record claim
   is corrected in `1235`.
7. **Minor**: the coverage step calls `python3`; the workflow-level
   permissions are narrowed per
   `2026-09-23-release-workflow-gating-and-permissions.md` in the same
   pass; `2026-09-23-release-workflow-tests-pass-with-the-behavior-broken.md`
   closes through the mutation tests above.

## Acceptance

- The coverage script passes against a changelog whose Unreleased section
  was renamed to the release heading, and against the real v0.9.0..HEAD
  history shape with merge-commit-only discovery.
- Behavior tests for the version script cover match, mismatch, missing
  `v`, pre-release, and `check-version-sync.sh` invocation.
- Mutation tests fail for each neutering edit the reviewer used.
- Every wheel is smoked with the stricter exit criteria.
- Full yellow gate at the head SHA; the runbook describes the draft-last
  flow; both issue files gain Resolved sections; a record lands.

## Review

- Design review: pending
- Code review: pending
