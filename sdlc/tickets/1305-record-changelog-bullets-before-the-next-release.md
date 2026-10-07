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

- No behavior changes; docs-only ticket.

## Proof

The coverage check passes with every landed ticket named.

## Priority note

P3: due before the next release tag, which has no date.

Note: hand-numbered. pm ticket new drew 2010 despite the numbering
ruling (see the message to the pm team, 2026-10-06); the reservation
was released and this ticket takes 1305.
