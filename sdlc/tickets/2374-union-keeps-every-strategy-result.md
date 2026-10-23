# 2374 — Union keeps every result its strategies find

Status: OPEN.

Milestone: 0.9.4

Priority: 0

## Outcome

The union strategy returns every PMID that any of its component strategies returns for the same query.

## Evidence

- Research test of the 0.9.4 practice artifact (run 39857815354, SHA-256 6dbc3a83...4739) on Blue, 2026-10-23, relayed by Jarvis in m4147. Plain PubMed ESearch for 'AKT1 E17K' (relevance sort) returns 192 PMIDs. `biomcp --json variant articles 'AKT1 E17K'` (union, --limit 50, 35 pages) returns about 1,386 PMIDs, yet 20 of PubMed's 192 never appear. Ian's ruling: the worst case is variant articles failing to return what a plain PubMed search finds.
- PMID 33303690 (PubMed rank 4, 'AKT1 E17K Inhibits Cancer Cell Migration by Abrogating beta-Catenin Signaling') is returned by --strategy annotation --limit 50 and by `search article -k 'AKT1 E17K' --limit 50`, but not by union over 35 pages. PMID 18783292 ('AKT1 (E17K) mutation in pancreatic cancer') is in search article but not in union.

## Proof

- Outside-in replay test: for AKT1 E17K, union's paged result is a superset of each component strategy's paged result. Name the step that drops them (dedup, identity verification, caps, deadline) and fix it.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks AKT1 E17K on the final build.
