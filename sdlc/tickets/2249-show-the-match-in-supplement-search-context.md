---
priority: 2
---

# 2249 — Show the match in supplement search context

Status: OPEN.

Milestone: 0.9.4

## Outcome

Each `article supplement-search` hit's context includes the matched string with bounded text around it.

## Evidence

Reproduced on 0.9.3: `biomcp --json article supplement-search 24451234 R1436Q` finds the right page-21 hit, but its context omits the matched string.

## Keeps

Keep hit location and ranking. Bound the context length.

## Proof

A fixture test shows the context containing R1436Q for that supplement page, and a match near the start or end of a page stays within bounds.
