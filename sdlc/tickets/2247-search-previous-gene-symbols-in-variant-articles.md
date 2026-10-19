---
priority: 2
---

# 2247 — Search previous gene symbols in variant articles

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search also queries a gene's previous official symbols and well-recorded aliases, labeled with where each came from, or states plainly that it searched only the current symbol.

## Evidence

Reproduced on 0.9.3: the ABCC8 R1437Q search never uses the older name SUR1. Europe PMC `SUR1 AND "R1437Q"` returns six PMIDs, including free PMC papers 10685980 and 9312191, against three for `ABCC8 AND "R1437Q"`.

## Keeps

Take alternative symbols only from a recorded source such as HGNC previous symbols, never a guess. Bound the number of extra queries within the deadline. Keep result order rules.

## Proof

A replay test shows the SUR1 query sent with its provenance and the extra papers returned, or the restriction stated in the output.
