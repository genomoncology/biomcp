# Make the changelog gate see every ticket and the workflow tests guard every step

From sdlc/issues/2026-09-24-release-gate-follow-ups-after-1234.md. The
reviewer ran a v0.9.1 dry run: the coverage gate rightly failed, but
for the wrong set of tickets, a bare number list still passes, and 17
of 18 workflow mutations outside the version-check job stay green.

## Problem

1. `scripts/check-changelog-coverage.py:12` discovers tickets only
   from merge subjects matching `tickets/NNNN-`. Eight merged tickets
   (1219, 1220, 1222, 1223, 1224, 1236, 1237, 1247) landed with other
   subjects or no merge commit, so the gate skips them silently.
2. The bullet parser (:92) accepts `- 1226, 1227, 1228` as a
   described bullet: the per-ticket marker strip leaves digit tokens
   and separators behind, and the leftover text passes the
   `remainder and not remainder.isdigit()` check. A bare number list
   describes nothing but passes.
3. The workflow provenance tests assert no-escape properties only for
   the version-check job. `continue-on-error` on wheel-smoke or
   docs-live, `|| true` on the floor check, dropping the ARM wheel
   build, `if: false` on the runtime floor smoke, deleting the
   not-found exit, and restoring a `release: published` trigger all
   leave the tests green.

## Design

1. Discover tickets as the UNION of two git scans between the
   previous stable tag and the target tag: `sdlc/records/NNNN-*.md`
   files added (`git diff --name-only --diff-filter=A <prev> <tag> --
   sdlc/records/`) AND the existing merge-subject scan
   (`tickets/NNNN-`). Neither alone is complete — a record can lag a
   merge (ticket 1246 merged with no record yet), and a merge subject
   can be rewritten or absent — so the gate fails closed on the union.
2. Bullet acceptance: after removing the ticket marker, also remove
   every remaining bare number token (and separators), then require
   the leftover text to contain at least three word characters in
   aggregate. `- 1226, 1227, 1228` then has no described text and
   fails; `- Fixed the wheel floor check (1246)` passes.
3. Extend the workflow tests to every step of pypi-build,
   wheel-smoke, and docs-live, mirroring the version-check
   assertions: no `run:` step carries `continue-on-error: true` (the
   two `setup-protoc` `uses:` steps keep their existing carve-out —
   protoc is optional, generated code is committed); no run string
   contains `|| true`, `|| :`, `; exit 0`, or `if: ${{ false }}`, and
   no step key is literally `if: false` or `if: ${{ false }}`; the
   matrix entries (runner, container, target, artifact name) are each
   pinned exactly; the trigger block contains `push:` with tags and
   does not contain `release:` (mind the YAML 1.1 quirk: `on:` parses
   as boolean `True` under safe_load); the floor-check step exists in
   both wheel legs; the ARM build and smoke legs exist with their
   runners. Three content pins the issue's mutations demand by name:
   the `return 1` after the panic message in the venv smoke, the
   not-found exit check after `drug adverse-events`, and the runtime
   floor-smoke step's `if:` condition pinned to
   `runner.os == 'Linux'`. The reviewer's mutation list in the issue
   is the test matrix: each named mutation must flip at least one
   assertion. Implement as data-driven cases over the parsed YAML so
   a new step in those jobs is covered without a new hand-written
   assertion.
4. The 15 missing changelog bullets are release-prep work, not this
   ticket: the gate is right and the bullets get written then. The
   release-prep checklist ticket is
   `sdlc/tickets/1253-release-preparation-checklist.md`, which lists
   them. The first-release branch (no previous tag: exit 0, "no
   previous release") stays as it is today.

## Acceptance

- A synthetic repo fixture (or a git-history-driven test) proves a
  ticket discovered only through `sdlc/records/` fails the gate
  without a bullet and passes with one.
- The bare-number bullet fails.
- Every mutation in the issue's list flips at least one test.
- Yellow gate green at the head SHA.

## Review

- Design review: pending
- Code review: ACCEPT with two P2s folded (real-git fixture;
  job-level escape assert) 2026-09-25
- Verification: yellow gate at 2159551c lint/test/spec OK; see
  `sdlc/records/1250-make-the-changelog-gate-and-workflow-tests-total.md`
