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
- Changes: `src/entities/article/variant_search.rs`, `src/entities/article/identity_verification.rs`, `src/cli/variant/articles.rs`, `tests/test_variant_article_transcript_aliases.py`, `tests/fixtures/article/**`, `CHANGELOG.md` and this ticket. Preserve transcript provenance once in the shared result classification and use it in CLI and MCP output.
- Proof: recorded replies for BRCA1 F1761I show alias-only papers such as 34829431 in the unverified alias group with the alias and transcript named; a paper that states a projecting accession, coordinates with a build, or a resolving rsID is verified; an alias that is a different valid MANE variant is labeled or dropped.

## Progress

- 2026-10-18 started

## Development brief

Restated outcome: preserve transcript-tagged retrieval aliases, show verified papers first, label alias-only matches as unverified with alias and source transcript, and exclude them from evidence summaries. Accept identity only through the recorded clinical ruling; retain the existing typed Gene rule.

Study found alias expansion flattens transcript provenance and Markdown conversion discards variant identity metadata. Planned outside-in regression invokes JSON and Markdown variant articles against an isolated recorded local fixture. Reproduction has not run: original research replies for PMID34829431 and its alias mapping are requested from the PM. Do not invent recorded evidence or claim a reproduced failure before the admitted command runs.
