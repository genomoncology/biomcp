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

## Recorded reproduction

The authorized fresh live capture ran on 2026-10-18 through the existing recorder using verified released v0.9.3. Job2185/1792358495704-88016 passed in28.199s. Command: `biomcp --json --no-cache variant articles 'BRCA1 F1761I' --strategy union --verify-identity --debug-plan --limit 10`; exit0. PMID34829431 appears first, matched only by BRCA1F71I and unverified with no observations;32957588 follows with c.211T>A, also unverified. All10visiblepapers are unverified. The capture retained84freshresponses, manifest, date, exactargv and outputs in owned experiment2079 pending review. BinarySHA25654f66769d9193c569d1b65756ad766eafd1a2c5e92d08f8c855cc8964e9a7145; archiveSHA256e637062c18c6692f3ec49c5847e69b29f6feee7f7909345970bd504d72174994.

PM decision: Confirmed conservative handling on2026-10-18. Label such hits "unverified alias match", name the alias and say "source transcript not stated by the provider" when provenance is missing. Name a transcript only when supplied by a provider. Keep alias-only matches out of verified evidence and never infer transcript association by zipping arrays. No positive verified-identity claim comes from this capture. Current-main RED using these recorded bytes is the next step before implementation.
