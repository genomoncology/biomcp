---
priority: 3
---

# 2185 — Judge variant aliases from other transcripts

Status: OPEN.

Milestone: 0.9.4

## Outcome

Variant article search follows the clinical ruling on transcript aliases: an alias-only match is verified only when the paper pins it to the same genomic change, and unverified alias matches appear in their own labeled group, never in evidence summaries.

- Evidence: research's spot check: `BRCA1 F1761I` expands to aliases such as `BRCA1 F71I` and `BRCA1 c.211T>A` from other transcripts, which pull papers matched only by those names, for example 34829431; identity checks mark them unverified.
- Starts from: `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`, the clinical review ruling.
- Changes: claims the alias expansion in `src/entities/article/variant_search.rs` and its tests, `CHANGELOG.md`, and the output that groups article results.
- Proof: recorded replies for BRCA1 F1761I show alias-only papers such as 34829431 in the unverified alias group with the alias and transcript named; a paper that states a projecting accession, coordinates with a build, or a resolving rsID is verified; an alias that is a different valid MANE variant is labeled or dropped.
