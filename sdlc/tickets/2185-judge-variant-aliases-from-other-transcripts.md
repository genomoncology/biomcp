---
priority: 3
---

# 2185 — Judge variant aliases from other transcripts

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 73da5c5, accept

## Outcome

Variant article search follows the clinical ruling on transcript aliases: an alias-only match is verified only when the paper pins it to the same genomic change, and unverified alias matches appear in their own labeled group, never in evidence summaries.

- Evidence: research's spot check: `BRCA1 F1761I` expands to aliases such as `BRCA1 F71I` and `BRCA1 c.211T>A` from other transcripts, which pull papers matched only by those names, for example 34829431; identity checks mark them unverified.
- Starts from: `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`, the clinical review ruling.
- Changes: `src/entities/article/variant_search.rs`, `src/entities/article/identity_verification.rs`, `src/cli/variant/articles.rs`, `tests/test_variant_article_transcript_aliases.py`, `tests/test_variant_article_exact_fulltext.py`, `tests/fixtures/article/**`, `CHANGELOG.md`, `sdlc/tickets/2226-resolve-explicit-paper-transcript-and-genomic-anchors.md` and this ticket. Preserve transcript provenance once in the shared result classification and use it in CLI and MCP output.
- Proof: recorded replies for BRCA1 F1761I show alias-only papers such as34829431 in the unverified alias group with their alias and provider-supplied transcript, or the approved missing-provenance wording. Existing real TP53/CHEK2 captures continue to prove the supported paper-linked same-allele CAID/rsID path. Preserve conservative identity decisions for unsupported anchors and keep alias-only papers outside verified evidence. General paper-text transcript/build-coordinate resolution belongs to2226.

## Progress

- 2026-10-18 started

## Development brief

Restated outcome: preserve transcript-tagged retrieval aliases, show verified papers first, label alias-only matches as unverified with alias and source transcript, and exclude them from evidence summaries. Accept identity only through the recorded clinical ruling; retain the existing typed Gene rule.

Study found alias expansion flattens transcript provenance and Markdown conversion discards variant identity metadata. Planned outside-in regression invokes JSON and Markdown variant articles against an isolated recorded local fixture. Reproduction has not run: original research replies for PMID34829431 and its alias mapping are requested from the PM. Do not invent recorded evidence or claim a reproduced failure before the admitted command runs.

## Recorded reproduction

The authorized fresh live capture ran on 2026-10-18 through the existing recorder using verified released v0.9.3. Job2185/1792358495704-88016 passed in28.199s. Command: `biomcp --json --no-cache variant articles 'BRCA1 F1761I' --strategy union --verify-identity --debug-plan --limit 10`; exit0. PMID34829431 appears first, matched only by BRCA1F71I and unverified with no observations;32957588 follows with c.211T>A, also unverified. All10visiblepapers are unverified. The capture retained84freshresponses, manifest, date, exactargv and outputs in owned experiment2079 pending review. BinarySHA25654f66769d9193c569d1b65756ad766eafd1a2c5e92d08f8c855cc8964e9a7145; archiveSHA256e637062c18c6692f3ec49c5847e69b29f6feee7f7909345970bd504d72174994.

PM decision: Confirmed conservative handling on2026-10-18. Label such hits "unverified alias match", name the alias and say "source transcript not stated by the provider" when provenance is missing. Name a transcript only when supplied by a provider. Keep alias-only matches out of verified evidence and never infer transcript association by zipping arrays. No positive verified-identity claim comes from this capture. Current-main RED using these recorded bytes is the next step before implementation.

## Current-main failing regression

Job2185/1792358727485-25955 replayed only the84recorded replies against the qualified072a57c38 executable, SHA2560fdd1309de4e56cf0011456a9c8120ebd329fdf5be04ea2a83f33df5174629ed. Command: `python -m pytest -q -s tests/test_variant_article_transcript_aliases.py` with BIOMCP_BIN set to that frozen executable and the existing prepared Python environment. Exit1 in1.246s; pytest reports3failedin1.12s and0replaymisses, with no live fallback or builds.

For PMID34829431, JSON fails `assert None == unverified_alias_match`; Markdown lacks the unverified alias group; compact MCP fails because actual match_reason is exact_variant instead of unverified_alias_match. The paper remains identity-unverified and matches only BRCA1F71I. The fixture contains actual captured provider data, not inferred transcript mappings.

Proceed with the shared row classification and provenance fix, compact MCP consumption, and Markdown grouping. Preserve the typed Gene rule and accept clinical identity only through the recorded ruling. Source transcript is named only from provider-paired metadata; missing provenance uses the exact PM-approved phrase. No new clinical inference is authorized.

## Bounded identity proof ruling

PM decision on2026-10-18: Preserve the existing identity verifier. Prove its supported same-allele path with the real TP53/CHEK2 captures plus the recorded unknown-alias regression. General paper-text anchor resolution is not added; unsupported anchors remain unverified. The separate priority3 ticket2226 covers explicit paper transcript/build-coordinate resolution and may move to a later release without blocking2185 or this release.

## Review result

Fresh review accepted source dc37ea4256c9df86d462da9e200aebcc2768310c after fixing an empty-search Markdown regression. The existing exact-fulltext fixture now asserts the retained no-articles message. Job2185/1792359386035-98769 reproduced that regression then passed formatting, Ruff, all-target routine Clippy, one CLI build, all9focused cases and quality checks. Qualified executableSHA25666c5be0f203a876bc3dc74f8464510bdc64e2b61f6d74ed64df00100bc53e2af. The reviewer independently compared all84tracked replies with the retained original capture and reused the passing real TP53/CHEK2 proofs.
