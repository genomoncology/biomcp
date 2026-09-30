---
ticket: 1265
head: b1393806
merge: cbe6233a chain
---

# 1265 — the changelog gate's record pattern

## What changed

The changelog coverage gate dropped real ticket records whose slug started with a digit or a capital, read dated notes as tickets, and its regression test exercised the pattern alone. The landing accepted any four-digit number with any slug, rejected date-shaped names outright, and drove `record_tickets()` end to end against a scratch git repo. The under-2000 cap that first landing kept was later removed entirely (see the follow-up note below).

## Known defect, recorded not hidden

The follow-up closed on 2026-09-30: the under-2000 cap is gone, and date-shape rejection is the whole rule — a date-shaped name (dddd-dd-dd, any year) never counts, and any other four-digit number does, including 2000 and above (the gate test plants tickets 2000 and 2027 counting).

## Evidence

- Branch head `b1393806` ("Fold the review pass-two findings"); yellow gates 21d42b2a and b1393806 passed on the gate host. The branch-head run has expired in the Actions API's retrievable window; no run ID exists to cite for it. The chain's final green state is 3d293ad7 → CI 36566765215 (success), and the 0.9.1-candidate main runs 36687060448 and 36693388098 are green with this code in them.
- Code review: none was recorded before the merge, and no reviewer has reviewed this ticket since; that is a fact, not a verdict. The raise-review mechanism that demands accepted reviews landed later (ticket 1269).
