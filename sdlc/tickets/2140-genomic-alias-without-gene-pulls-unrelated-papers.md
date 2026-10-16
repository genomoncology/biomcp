---
flow: build
priority: 2
deps: []
---

# 2140: Tie genomic aliases to the gene in variant article search

Status: OPEN.

Milestone: 0.9.3

## Outcome

A variant article search keeps a paper matched only by a genomic coordinate alias when the paper is about the requested gene, and drops it otherwise.

## Evidence

Reported 2026-10-16 by a literature probe of 0.9.2. This reverses the earlier reading that wrong-variant matches were not real.

- `biomcp --json variant articles "SAMHD1 I448T" --limit 50`, then `--offset 50`: `chr20:g.35533834A>G` is the only match for 28 of 100 results, including papers on cotton (28794480) and citrus (41894487).
- For CDKN2A, `chr9:g.21971188G>A` is the only match for 19 of 100 results.

## Work

- Reproduce on main first.
- Send genomic aliases with the gene symbol, or keep an alias-only hit only when the paper also names the gene. Pick one and record why in this ticket.

## Acceptance

1. A recorded-reply test of the SAMHD1 I448T search drops the two off-topic papers and keeps the papers that name SAMHD1; it fails on current main.
2. The changelog carries a bullet for this ticket.
