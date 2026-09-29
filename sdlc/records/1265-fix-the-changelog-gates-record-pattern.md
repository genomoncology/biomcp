---
ticket: 1265
head: b1393806
merge: cbe6233a chain
---

# 1265 — the changelog gate's record pattern

## What changed

The changelog coverage gate accepted ticket references as bare four-digit numbers with any following slug. The gate now treats a record reference as four digits whose value stays under 2000 followed by an arbitrary slug, and dated records (the `2026-09-27-...` shape) ride a separate branch the gate already handled. A regression test drives the gate's record parser end to end against a scratch changelog.

## Known defect, recorded not hidden

The 2026-09-29 review flagged the under-2000 cap as a defect: it is a magic number standing in for "not a date." The honest rule is "reject date-shaped names outright." The cap stays until the gate is rewritten with that explicit rule; ticket 1275's follow-up list carries it.

## Evidence

- Branch head `b1393806` ("Fold the review P2s"); the branch's CI run at record time is beyond the retrievable window of the Actions API, so no run ID is cited here. The merge chain reached main; the runs readable at the merge are recorded in the 1269 entry.
- Code review: none recorded before the merge; the ticket file carries the round's verdict lines.
