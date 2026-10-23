# 2376 — Variant articles pages over a fixed pool

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

Paging `variant articles` walks one fixed candidate pool: each PMID appears on exactly one page, and paging to the end returns the whole pool every time.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- Union's reported candidate pool changed between pages during the run. The 2006 AKT mutations paper appeared near rank 390 in one run and was missing from another, longer run. Related: 2365 found CHEK2 G167R paging returning 0 rows with has_more=true.

## Proof

- Outside-in replay test: paging AKT1 E17K to the end twice gives the same ordered PMID list with no duplicates and no gaps, and the pool size is the same on every page.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.
