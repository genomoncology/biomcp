# Rework the release gates so v0.9.1 can pass them

Filed from `sdlc/issues/2026-09-23-release-gates-from-1233-block-v0-9-1-and-test-nothing.md`; revised after two REJECT design reviews (2026-09-23). Blocks 0.9.1.

## Design

### Trigger and publication order

1. The workflow triggers on `push: tags: ['v*']` and keeps
   `workflow_dispatch` solely for published-release container backfills.
   `TAG` is `inputs.tag || github.ref_name` (dispatch names its tag
   explicitly; a push has no inputs, so `ref_name` applies).
2. A `create-draft` job runs `if: github.event_name == 'push'`, needs
   `version-check`, and creates the GitHub release as a draft so
   `gh release upload` has a target. `build` needs
   `[version-check, create-draft]`; its upload step's guard moves from
   the `release` event to `push`.
3. A final `publish-release` job needs
   `[build, pypi-publish, homebrew-tap, container-publish, docs-live]`,
   carries job-level `contents: write`, and runs `gh release edit
   --draft=false` plus the container `latest` move (the unqualified
   release lookup is only valid once the release is public).
4. Guarantee, stated precisely: the GitHub release, its assets, and the
   container `latest` pointer stay non-public until every check passes.
   PyPI and the Homebrew tap publish after their own gates and can
   briefly precede the GitHub release if a later job fails — the same
   channel exposure as today, now recorded here instead of implied away.
5. Every job-level `if` is enumerated here and pinned by tests:
   - `version-check`: none (runs on both triggers).
   - `create-draft`: `github.event_name == 'push'`.
   - `build`: `github.event_name == 'push'`.
   - `pypi-build`: `github.event_name == 'push'`.
   - `wheel-smoke`: `github.event_name == 'push'`.
   - `docs-live`: none (runs on both triggers; backfills need the gate).
   - `pypi-publish`: `github.event_name == 'push'`.
   - `homebrew-tap`: `github.event_name == 'push'`.
   - `container-publish`:
     `!cancelled() && ((github.event_name == 'push' && success()) || (inputs.container_only == true && needs.version-check.result == 'success' && needs.docs-live.result == 'success' && needs.create-draft.result == 'skipped' && needs.build.result == 'skipped' && needs['wheel-smoke'].result == 'skipped'))`.
     Truth table: push with every need green runs; push with any failed
     need is blocked by the explicit `success()`; dispatch with
     `container_only: true` runs only when `version-check` and
     `docs-live` succeeded and every push-only need is skipped; dispatch
     without `container_only` is blocked. `success()` sits inside the
     push branch because at job level it is false whenever a needed job
     was skipped, not only failed.
   - `publish-release`: `github.event_name == 'push'`.
   Dispatch contract, recorded: a dispatch with `container_only: true` runs
   `version-check`, `docs-live`, and `container-publish` only; a dispatch
   without it runs `version-check` and `docs-live` only, as a check-only
   dry run that publishes nothing. `always()` appears nowhere; every
   needs-result condition pairs `!cancelled()` with branch-local success
   requirements.
6. No `skip-existing` anywhere: release asset upload keeps `--clobber` so a
   rerun replaces partial uploads (ticket 1222's decision), and PyPI
   publish fails loudly on a version collision.

### Version and changelog gates

7. `scripts/check-release-versions.py` requires a `^v` tag, compares it
   to the committed Cargo and pyproject versions, and runs
   `scripts/check-version-sync.sh` from the tag ref (`fetch-depth: 0`).
   Behavior tests cover match, mismatch, missing `v`, and stable-only
   rejection with a clear message; `check-version-sync.sh` keeps its
   stable/`-dev.N` forms. Recorded decision: stable tags only; rc support
   becomes its own ticket if ever needed.
8. `scripts/check-changelog-coverage.py` reads the `## <tag version>`
   section first (stripping `v`, allowing the date suffix, escaping the
   version) and falls back to `## Unreleased`, matching to the next
   heading or end of file, failing clearly when neither exists.
9. Tickets come only from `Merge ... tickets/NNNN-` commit subjects via
   `git log <previous>..<tag>` on the full checkout; any-length numbers;
   each needs a described bullet; an internal-only marker bullet covers
   tickets with no user-visible change.

### Event gates for publishers

10. `pypi-publish` and `homebrew-tap` gate on
   `github.event_name == 'push'` (replacing the 1229 `release` gate), so
   a dispatch still cannot reach PyPI or the tap on any input.

### Smoke and permissions

11. The wheel smoke runs as a matrix over all four built wheels; Unix
    legs use `bin/biomcp`, Windows uses `Scripts/biomcp.exe`. Deep-path
    commands exit 0; the not-found adverse-event command exits with its
    exact expected code and text; exit 101 and any crash fail. Platform
    legs share live providers; a transient provider outage fails the
    release (accepted, fail-closed).
12. The permissions package lands in full: top-level `permissions: {}`;
    per-job grants — `contents: write` for `create-draft` and the
    asset-uploading `build`; `contents: read` for `version-check`,
    `pypi-build`, `docs-live`, and `homebrew-tap` (tap writes go through
    `HOMEBREW_TAP_TOKEN`); `id-token: write` plus `contents: read` for
    `pypi-publish`; `packages: write` plus `contents: read` for
    `container-publish`; `contents: write` for `publish-release`; no
    permissions for `wheel-smoke`. Also: the Homebrew push-event gate,
    tag-resolution retry, workflow-level concurrency, action SHA
    pinning, and the PyPI trusted-publisher confirmation.

### Expected needs adjacency (all edges mutation-tested)

```
version-check:            (none)
create-draft:             version-check
build:                    version-check, create-draft
pypi-build:               version-check
wheel-smoke:              pypi-build
docs-live:                version-check
pypi-publish:             pypi-build, wheel-smoke, docs-live
homebrew-tap:             build, docs-live, wheel-smoke
container-publish:        build, docs-live, version-check,
                          wheel-smoke, create-draft
publish-release:          build, pypi-publish, homebrew-tap,
                          container-publish, docs-live
```

13. Workflow mutation tests prove: every gate step has no
    `continue-on-error`, no `if:` escape, no `|| true`; removing ANY edge
    above fails a test, including the dispatch skipped-draft route; the
    mutation list covers the full adjacency exactly, one test per edge;
    and dropping the branch-local `success()`, any
    `needs.*.result == 'success'` clause, or the `inputs.container_only`
    clause from the `container-publish` condition fails a test.

## Acceptance

- The coverage script passes against a renamed-heading changelog and an
  end-of-file Unreleased section, discovering tickets from merge subjects
  only.
- The version script behavior tests pass, including the stable-only
  rejection.
- Mutation tests fail for each neutering edit and for each removed edge.
- Every wheel is smoked with the stricter exit criteria.
- Full yellow gate at the head SHA; the runbook describes the tag-push
  draft-last flow and the dispatch backfill; this issue file, the
  permissions file, and the tests-pass-with-behavior-broken file gain
  Resolved sections; a record lands.

## Review

- Design review: REJECT six times, ACCEPT on the seventh 2026-09-23
  (gpt-5.6-sol, medium). First: draft-creation job, latest-guard-under-
  draft, dispatch survival, permissions package, rc contradiction,
  needs-edge mutations. Second: `inputs.tag ||` order, push-only
  publisher gates, the false nothing-public claim, per-job permission
  grants, explicit skip semantics, and an exact adjacency list. Third:
  `create-draft` in container-publish's adjacency (direct needs only),
  full `if` enumeration in the ticket, `pypi-build` contents:read,
  `!cancelled()` everywhere, and the skip-existing choice. Fourth:
  `container_only` required in the dispatch clause, and explicit
  `success()` beside `!cancelled()` so a failed need can never publish
  on push. Fifth: `success()` is false for skipped needs, so it is
  branch-local (push only) and the dispatch branch names each required
  success and each required skip; numbering fixed. Sixth: caught that the
  fifth-round condition edit never landed. Seventh: ACCEPT; one P2
  bookkeeping note, fixed here.
- Code review: pending
