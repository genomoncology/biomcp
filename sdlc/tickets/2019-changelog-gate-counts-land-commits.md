# 2019 — Changelog gate counts Land commits

Status: OPEN.

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
