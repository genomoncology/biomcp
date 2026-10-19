---
priority: 3
---

# 2226 — Resolve explicit paper transcript and genomic anchors

Status: OPEN.

Milestone: 0.9.4

## Outcome

Recognize a paper's explicit transcript accession and version or genomic coordinates with a named build as verified identity only when authoritative resolution proves the same requested allele, following the recorded clinical ruling. Unsupported or unresolved anchors remain unverified. This work can move to a later release without blocking the demonstrated alias-label correction or the current release.

- Changes: `src/entities/article/variant_search.rs` `src/entities/article/identity_verification.rs` `tests/test_variant_article_paper_anchors.py` `tests/fixtures/article/paper-anchors/*.json` and the explicit-anchor example in `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`. The exact capture files are car-nm_007294.3-c.3748g-t.json, car-nm_007297.4-c.3607g-t.json, car-nm_007297.4-c.3607g-c.json and capture.json (source URL, UTC time, HTTP status and response hash). Reuse the existing actual paper capture; an explicitly simulated provider 503 is only a refusal control, never a recorded outage. Additional paths must be named before writing. Implement only a demonstrated mechanism after the positive current-main reproduction.


## Evidence

2185's fresh recorded regression proves unknown-provenance alias-only papers must stay unverified. The existing verifier has real TP53 and CHEK2 paper-linked CAID/rsID proofs, but it does not generally extract and resolve paper-text transcript or build-coordinate anchors. The PM separated this additional mechanism from the reproduced label/provenance defect on2026-10-18. See `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`.

## Changes

`src/entities/article/variant_search.rs`, `src/entities/article/identity_verification.rs` and focused recorded identity fixtures and tests. Review the exact boundaries before implementation; do not build a general text interpretation framework.

## Keeps

Preserve the existing typed Gene rule, conservative unknown-provenance handling, and supported paper-linked CAID/rsID path. Do not infer transcript associations from array positions or accept coordinates without a genome build. Obtain clinical review for any new interpretation beyond the recorded ruling.

## Proof

Use recorded authoritative same-allele resolution and actual paper anchors. For each added mechanism, show an outside-in positive same-allele case and a different-allele or unsupported-anchor control. Keep an unresolved anchor unverified and outside verified evidence summaries. Run only the affected checks through the existing build job.

## Study and proof boundary

Restated outcome: verify a literal versioned paper anchor only when its authoritative allele is the requested allele; preserve typed Gene identity, provenance and conservative unresolved behavior.

Current-main control: `08b74a340658f2c30e158fd00459330baebacd74`, job `2226/1792409910934-66270`, ran the existing `tests/test_variant_article_transcript_aliases.py` against the frozen qualified executable (SHA256 `886e80635de11125329d039da533c4e1ca7ee478936b4c0eb330bcf05d138b4c`), three command/MCP cases passed with zero replay misses. PMID34829431 remains unverified with no observations and the approved missing-provenance wording. This is a control, not a failing positive anchor reproduction.

The actual PMID32272925 capture contains literal `NM_007294.3:c.3748G > T` in paragraph43 and table26, but no authoritative paired response was recorded. A separate clinical reviewer confirmed the explicit prose anchor and live ClinGen equivalence with `NM_007297.4:c.3607G>T`, both `CA002402`, on2026-10-19. The cited ruling records sources. Capture those responses with provenance, then demonstrate a positive same-allele failure on current main before product changes. Do not substitute a fabricated positive. Different-allele and unresolved controls must remain unverified. No qualifying build-coordinate paper has been identified, so do not implement an unproved coordinate interpretation mechanism.

## Progress

- 2026-10-19 started
