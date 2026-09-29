# Fix the changelog gate's record pattern

From sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md
(Release, second bullet).

## Problem

The dated-name fix traded one wrong rule for three more:
`2026-q3-review.md` and `2026-sept-review.md` still parse as ticket
2026; a real ticket record whose slug starts with a digit is
dropped; uppercase slugs are dropped. The regression test checks
the pattern alone and its fake git harness does nothing.

## Fix

Ticket numbers in this repo are four digits under 2000; years are
2026 and up. The pattern keeps a match only when the parsed number
falls in the ticket range, with any slug following. The regression
test drives `record_tickets()` end to end against a real records
directory shape.

## Review

- Design review: n/a (the review file named the rule)
- Code review: ACCEPT with three report-only P2s (dead helper
  removed; the sample renamed five-digit; this verification line
  filled) 2026-09-28
- Verification: yellow gate at 21d42b2a — lint, test, spec,
  stress OK, zero failed lines; pytest 20 passed locally
