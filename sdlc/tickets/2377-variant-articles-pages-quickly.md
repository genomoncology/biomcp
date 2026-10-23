# 2377 — Variant articles pages quickly

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

A 50-row page of `variant articles` returns within seconds on a warm cache, and later pages reuse the fixed pool instead of rebuilding it.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- About 52 to 57 s per 50-row page on Blue, so about 10 minutes for 400 papers; PubMed ESearch returns all 192 in under a second.

## Proof

- Route-level timing names where each page's time goes. Measured on the M5 against the same query: page 2 onward reuses the pool from 2376 and returns well under the first page's time; record the before and after.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.
