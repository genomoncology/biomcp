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

## Positive reproduction before implementation

Authoritative capture job `2226/1792410302044-12663` recorded HTTP200 for the paper anchor and alternative transcript, both `CA002402` with response SHA256 `fcc99398fadd92ab1c56fe51c7b0ca1cc82f623f44d506796f5e492b6bf08fae`. The different-allele `NM_007297.4:c.3607G>C` returned `CA10594475`, response SHA256 `1d57dcfed34bb6c4d7bc89f3b8e29867abf2b0224851aa4bdfd18274cc9bf1a2`. Capture metadata records actual URLs, time, status and hash.

RED job `2226/1792410374016-38223` applied the exact test/capture patch to frozen main `08b74a340658f2c30e158fd00459330baebacd74` and ran `python -m pytest tests/test_variant_article_paper_anchors.py` through the frozen qualified CLI. Result: one failed, two passed. Same-allele PMID32272925 was unverified with empty observations instead of confirmed. Different and unresolved controls stayed unverified with empty confirmed-only results. No unknown fixture requests. The actual paper bytes are unchanged; test discovery is explicitly synthetic and the provider503 is explicitly simulated. Current main does not call that unresolved provider route because it lacks the mechanism, so actual failure-path handling must be asserted after implementation.

Implement only the demonstrated literal-versioned-transcript mechanism with bounded authoritative resolution, retained paper locator and provenance, plus active different-allele/unresolved controls. No unsupported build-coordinate parser or general text framework. Preserve typed Gene requirements and conservative summaries.

## Progress

- 2026-10-19 started
