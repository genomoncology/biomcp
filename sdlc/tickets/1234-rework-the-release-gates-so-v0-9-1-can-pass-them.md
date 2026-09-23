# Rework the release gates so v0.9.1 can pass them

Filed from `sdlc/issues/2026-09-23-release-gates-from-1233-block-v0-9-1-and-test-nothing.md` and revised after a REJECT design review (2026-09-23). Blocks 0.9.1.

## Design

### Trigger and publication order

1. The workflow triggers on `push: tags: ['v*']` and keeps
   `workflow_dispatch` solely for published-release container backfills.
   `TAG` derives per trigger: `github.ref_name || inputs.tag`.
2. A `create-draft` job (needs `version-check`) creates the GitHub release
   as a draft, so `gh release upload` has a target before `build` runs.
   The upload step's event guard moves from `release` to `push`.
3. A final `publish-release` job runs `gh release edit --draft=false` and
   needs every publisher: `build` (assets), `pypi-publish`,
   `homebrew-tap`, `container-publish`, and `docs-live`. It carries
   job-level `contents: write` after the top-level permissions are
   narrowed. No artifact is public before every check passes.
4. The container `latest` move leaves `container-publish` and lands in
   `publish-release`, after the release is public: the current unqualified
   `gh release view` guard reads the prior published release while the
   draft exists and would skip moving `latest` forever. The same
   latest-release check runs there against the now-public release.
5. Dispatch backfills keep the `container_only` contract; no draft is
   created (the release already exists).

### Version and changelog gates

6. `scripts/check-release-versions.py` requires a `^v` tag, compares the
   tag to the committed Cargo and pyproject versions, and runs
   `scripts/check-version-sync.sh` from the tag ref (the workflow checks
   out with `fetch-depth: 0` because the script inspects reachable tags).
   Behavior tests cover match, mismatch, missing `v`, and stable-only
   rejection: a pre-release tag fails with a clear message, and
   `check-version-sync.sh` keeps accepting only stable or `-dev.N` Cargo
   forms. Recorded decision: the release workflow supports stable tags
   only; pre-release support becomes its own ticket if ever needed.
7. `scripts/check-changelog-coverage.py` reads the `## <tag version>`
   section first (stripping `v`, allowing the date suffix, escaping the
   version) and falls back to `## Unreleased`, matching up to the next
   heading or end of file, with a clear failure when neither exists.
8. Tickets come only from `Merge ... tickets/NNNN-` commit subjects via
   `git log <previous>..<tag>` on the full checkout; numbers of any
   length; each needs a described bullet; an explicit internal-only
   marker bullet covers tickets with no user-visible change.

### Smoke and permissions

9. The wheel smoke runs as a matrix over all four built wheels; Unix legs
   use `bin/biomcp`, Windows uses `Scripts/biomcp.exe`. Deep-path
   commands exit 0; the not-found adverse-event command exits with its
   exact expected code and text; exit 101 and any crash fail. Platform
   legs share live providers, so a transient provider outage fails the
   release (accepted, fail-closed).
10. The permissions package from
    `2026-09-23-release-workflow-gating-and-permissions.md` lands in the
    same pass, all of it: the Homebrew release-event gate, upload
    `skip-existing` semantics, `!cancelled()` where a needed-job chain
    requires it, tag-resolution retry, workflow-level concurrency, action
    SHA pinning, and the PyPI trusted-publisher confirmation.

### Tests

11. Workflow mutation tests prove each gate: every gate step has no
    `continue-on-error`, no `if:` escape, no `|| true`; removing any
    required direct or transitive publish-dependency edge (`version-check`,
    `wheel-smoke`, `create-draft`, `publish-release` needs) fails a test;
    each assertion is proven by mutating the workflow inside the test.

## Acceptance

- The coverage script passes against a changelog whose Unreleased section
  was renamed to the release heading and against an Unreleased section at
  end of file, and discovers tickets from merge subjects only.
- The version script behavior tests pass, including the stable-only
  rejection message.
- Mutation tests fail for each neutering edit the reviewer used and for
  each removed needs edge.
- Every wheel is smoked with the stricter exit criteria.
- Full yellow gate at the head SHA; the runbook describes the tag-push,
  draft-last flow and the dispatch backfill; both issue files (this one
  and the permissions file) gain Resolved sections; a record lands.

## Review

- Design review: REJECT 2026-09-23 (gpt-5.6-sol, medium) — six findings:
  tag-push with an explicit draft job (not a bare tag trigger); the
  container latest guard breaks under drafts; keep dispatch backfills;
  the permissions package was not fully covered; check-version-sync.sh
  rejects rc forms the ticket promised; mutation coverage missed the
  needs edges. All folded into this revision; second review pending.
- Code review: pending
