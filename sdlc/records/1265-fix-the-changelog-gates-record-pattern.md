---
ticket: 1265
head: b1393806
merge: cbe6233a chain
---

# 1265 — the changelog gate's record pattern

## What changed

The changelog coverage gate accepted ticket references as bare four-digit numbers with any following slug. The gate now treats a record reference as four digits whose value stays under 2000 followed by an arbitrary slug, and dated records (the `2026-09-27-...` shape) ride a separate branch the gate already handled. A regression test drives the gate's record parser end to end against a scratch changelog.

## Known defect, recorded not hidden

The follow-up closed on 2026-09-30: the under-2000 cap is gone, and date-shape rejection is the whole rule — a date-shaped name (dddd-dd-dd, any year) never counts, and any other four-digit number does, including 2000 and above (the gate test plants tickets 2000 and 2027 counting).

## Evidence

- Branch head `b1393806` ("Fold the review pass-two findings"); yellow gates 21d42b2a and b1393806 passed on the gate host. The branch-head run has expired in the Actions API's retrievable window; no run ID exists to cite for it. The chain's final green state is 3d293ad7 → CI 36566765215 (success), and the 0.9.1-candidate main runs 36687060448 and 36693388098 are green with this code in them.
- Code review: none was recorded before the merge, and no reviewer has reviewed this ticket since; that is a fact, not a verdict. The raise-review mechanism that demands accepted reviews landed later (ticket 1269).
