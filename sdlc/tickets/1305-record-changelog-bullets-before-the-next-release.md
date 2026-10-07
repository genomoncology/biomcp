# 1305 — Record changelog bullets before the next release

Status: OPEN.
Milestone: 0.9.2

## Outcome

Every landed ticket that owes a CHANGELOG bullet carries one before any
release tag is cut, so the changelog-coverage release gate passes on the
first try.

## Evidence

Tickets 1292, 1297, 1301, 1302, and 1303 each recorded a deferred
CHANGELOG bullet in their build status. The release gate enforces
coverage at tag time (tools/check-changelog-coverage.py, Unreleased
section convention). Amendment 2026-10-07, from the lane's live gate
run: the gate counts every ticket merged since v0.9.1, not only the
deferred ones, so the bullet set is widened to the truthful full set —
code bullets for 1290, 1293, 1294, 1295, 1296, 1298, and 1304 from
their land commits, and one Internal bullet covering 1287, 1288, and
1289 (sweep, issue filing and diagnosis, planning; no user-visible
change). The original evidence undercounted; the lane decision record
is in the build report.

## Change detail

1. Write the five bullets in `CHANGELOG.md` under the Unreleased
   heading, one sentence each in the house voice.
2. Run the coverage check against the landed set.

## Keeps

- No behavior changes. The branch also generalizes one assertion in
  tests/test_docs_changelog_refresh.py so the suite accepts the
  returning Unreleased section, so the branch is not strictly
  docs-only and CI runs the full suite on it; the merge bar is the
  branch's finished-green full run.

## Proof

The coverage check passes with every landed ticket named.

## Build status

- Built on branch `tickets/1305-changelog-bullets`, commits 361340002,
  4eef81a8a, fdc996c2d, plus the internal-bullet fold ba7abe31e,
  2026-10-07.
- Code review: FIX 2026-10-07 (fresh reviewer, fix round). P1: the
  branch is not docs-only (it carries the test generalization from the
  first commit) and the fix report's docs-only classification claim was
  false — confirmed by diff against base 739447590d; the merge bar is
  the full green run, recorded above in Keeps. P2: no bullet named
  1305 itself, which the gate demands once 1305 lands as a Land commit
  (ticket 2019's fixed gate) — folded as the internal-bullet clause in
  ba7abe31e. All bullet truthfulness findings verified correct: 1299,
  1306 against their landing merges; 1300 and 1291 against their branch
  tips with re-read obligations at their final tips. The 1293 bullet's
  honest joint credit confirmed.

## Priority note

P3: due before the next release tag, which has no date.

Note: hand-numbered. pm ticket new drew 2010 despite the numbering
ruling (see the message to the pm team, 2026-10-06); the reservation
was released and this ticket takes 1305.
