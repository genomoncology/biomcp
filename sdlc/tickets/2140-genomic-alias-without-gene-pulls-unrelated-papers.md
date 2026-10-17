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

- `biomcp --json variant articles "SAMHD1 I448T" --limit 50`, then `--offset 50`: `chr20:g.35533834A>G` is the only match for 28 of 100 results, including a paper on cotton (28794480).
- For CDKN2A, `chr9:g.21971188G>A` is the only match for 19 of 100 results, including a paper on citrus (41894487) on the second page.

## Work

- Reproduce on main first.
- Send genomic aliases with the gene symbol, or keep an alias-only hit only when the paper also names the gene. Pick one and record why in this ticket.

## Reproduction on main

Reproduced 2026-10-17 on current main `ec4f2e309` (pm job `1792206086379-58813` on
the macOS gate host; the job builds main's CLI with
`cargo build --no-default-features --bin biomcp` and runs the live search).

- `./target/debug/biomcp --json variant articles "SAMHD1 I448T" --limit 50`, then
  `--offset 50`: page 1 keeps 17 rows whose only match is `chr20:g.35533834A>G`, page 2
  keeps 6 more. The cotton paper (28794480, "Chromosome structural variation of two
  cultivated tetraploid cottons...") appears on both pages with provenance
  `route=exact_lexical, source=pubmed, matched_alias=chr20:g.35533834A>G` (and the same
  alias through Semantic Scholar on page 1).
- `./target/debug/biomcp --json variant articles "CDKN2A A57V" --limit 50`, then
  `--offset 50`: the citrus paper (41894487, "Highly heterozygous Citrus
  changshan-huyou...") appears on page 2 with the only match `chr9:g.21971188G>A`
  (native position 5; 3 coordinate-only rows on page 1 and 8 on page 2).

The probe's CDKN2A input spelling was not recorded; `CDKN2A A57V` resolves to the
reported alias `chr9:g.21971188G>A` (rs372266620, c.170C>T, p.Ala57Val), so the tests
use it. Provider counts drift daily (17+6 vs the probe's 28; 3+8 vs 19); the defect is
unchanged. PubMed tokenizes the bare coordinate to `"chr20"[All Fields] AND "G"[All
Fields]`, so any paper naming the chromosome matches.

## Acceptance

1. A recorded-reply test of the SAMHD1 I448T search drops the cotton paper and keeps the papers that name SAMHD1; it fails on current main.
2. The same holds for the CDKN2A search and the citrus paper.
3. The changelog carries a bullet for this ticket.
