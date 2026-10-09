# 2019 — Changelog gate counts Land commits

Status: complete.

Milestone: 0.9.2

## Outcome

`scripts/check-changelog-coverage.py` sees every ticket that landed since the previous stable tag, whatever shape the landing took, and fails the tag when one has no changelog bullet.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 4).

- The gate counts a ticket only from a `Merge ... tickets/NNNN-` subject (`scripts/check-changelog-coverage.py:12`) or a new ticket-numbered `sdlc/records/` file (`:17`). Tickets now land as `Land NNNN: ...` and record their build in the ticket file.
- On main at `37631c357` the gate sees 11 tickets. It cannot see 1293, 1299, 1302, 1303, 1304 or 1306.
- The reviewer merged 1300, 1305 and 1291 in the planned order with `Land NNNN:` subjects, tagged v0.9.2 in a scratch clone, and ran the gate. It printed "covers all 11 tickets" and exited 0. 1299, 1306, 1300 and 1291 would ship with no bullet.
- `scripts/ci-classify-push.sh:46` treats `CHANGELOG.md` as docs-only, but `tests/test_docs_changelog_refresh.py` reads it, so a changelog-only branch push skips the test that checks it.

- Starts from: ticket 1233, which built the gate.
- Keeps: the date-shaped record exemption and the stable-tag rule.
- Changes: also count `Land NNNN:` first-parent subjects on the release range, and any ticket whose status moved to complete in the range. Route `CHANGELOG.md` pushes through the changelog test.
- Proof: a scratch-tag test over a range containing a `Land NNNN:` merge with no bullet fails the gate. On main after 0.9.2's queue lands, the gate lists 1290 through 1306 plus this round's tickets.
- Defers: nothing.

## Build status

- Built on branch `tickets/2019-changelog-gate-counts-land-commits`,
  commit 99f2f87e5, 2026-10-07.
- Code re-review (follow-up delta 99f2f87e5..3bbc67d48): ACCEPT 2026-10-07, fresh reviewer. Verified: the status-complete source matches pm's token grammar exactly (including the decimal-period exception and 2010's real absorbed-status line); archive/ and drafts/ can never match; the six tests cover every claimed behavior; ci-classify routing is enforced end to end by the real behavioral tests; the +1 real-main delta is exactly 2010; no private names. Two report-only P2s: a pre-existing vacuous allow-list pin in test_ci_workflow_contract.py (the behavioral tests carry the enforcement), and the status mirror being narrower than pm's full ticket seam (fail-open only; every live ticket uses the bare form).
- Code review: ACCEPT 2026-10-07 for the lane's commit (the
  Land-subject union, six new tests — the builder's report said seven,
  six is correct —, real-main proof demanding exactly the 17-ticket
  list with finding 4's six as the delta, release-process doc match,
  no private names). Three P2s, none blocking the commit: the report's
  test count, an all-subjects scan broader than the Changes' first-
  parent wording (fail-closed direction, accepted), and the two
  remaining Changes items — status-complete counting and ci-classify
  CHANGELOG routing — which stay on this ticket and are being finished
  on the lane before landing.

## Landing

- Landed 2026-10-07 as merge ad39a9c5d. Process deviation, recorded
  honestly: the merge reached main through the queue owner's
  grammar-fix push (50c19bb35) before a green merged-tree run existed
  — the first landcheck run failed on the queue owner's own
  out-of-grammar review lines, and the fix push carried the merge
  with it. The verifying run then executed on the exact main tip
  (tickets/landcheck-2019 at 50c19bb35): completed, success. Every
  later landing runs its merged-tree CI before main moves.
