---
priority: 3
---

# 2226 — Resolve explicit paper transcript and genomic anchors

Status: OPEN.

Milestone: 0.9.4

## Outcome

Recognize a paper's explicit transcript accession and version or genomic coordinates with a named build as verified identity only when authoritative resolution proves the same requested allele, following the recorded clinical ruling. Unsupported or unresolved anchors remain unverified. This work can move to a later release without blocking the demonstrated alias-label correction or the current release.

## Evidence

2185's fresh recorded regression proves unknown-provenance alias-only papers must stay unverified. The existing verifier has real TP53 and CHEK2 paper-linked CAID/rsID proofs, but it does not generally extract and resolve paper-text transcript or build-coordinate anchors. The PM separated this additional mechanism from the reproduced label/provenance defect on2026-10-18. See `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`.

## Changes

`src/entities/article/variant_search.rs`, `src/entities/article/identity_verification.rs` and focused recorded identity fixtures and tests. Review the exact boundaries before implementation; do not build a general text interpretation framework.

## Keeps

Preserve the existing typed Gene rule, conservative unknown-provenance handling, and supported paper-linked CAID/rsID path. Do not infer transcript associations from array positions or accept coordinates without a genome build. Obtain clinical review for any new interpretation beyond the recorded ruling.

## Proof

Use recorded authoritative same-allele resolution and actual paper anchors. For each added mechanism, show an outside-in positive same-allele case and a different-allele or unsupported-anchor control. Keep an unresolved anchor unverified and outside verified evidence summaries. Run only the affected checks through the existing build job.
