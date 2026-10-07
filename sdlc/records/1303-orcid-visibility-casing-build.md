# 1303 build record — ORCID's real visibility casing accepted

Landed 2026-10-06, merge 3f5afdc2f. Review verdicts and change detail
live on sdlc/tickets/1303-accept-orcids-real-visibility-casing.md. Closes
external issue #289 (reporter answered; the fix ships in the next
release).

Outcome: `get author` and `author papers` read ORCID's lowercase
`public` visibility values; an `orcid:` lookup no longer fails and
papers no longer silently return zero works. Root cause: ORCID sends
lowercase where the schema declared title case. Gates: branch CI
green, yellow gates rc=0. The adversarial review held the fix against
a live read (reverting fails 10 tests). Record backfilled 2026-10-07
under ticket 2020.
