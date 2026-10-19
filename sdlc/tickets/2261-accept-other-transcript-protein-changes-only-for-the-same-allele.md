---
priority: 1
---

# 2261 — Accept other-transcript protein changes only for the same allele

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `sdlc/tickets/2251-report-publisher-elided-references-as-empty.md`, `sdlc/tickets/2252-make-variant-articles-markdown-match-its-json.md`, `sdlc/tickets/2253-document-full-text-access-groups-and-reason-codes.md`, `sdlc/tickets/2254-fix-article-help-lists-and-condition-names.md`, `sdlc/tickets/2256-bound-only-the-mcp-handshake-wait-by-the-startup-timer.md`, `sdlc/tickets/2257-route-every-test-cache-root-through-the-shared-guard.md`, `src/entities/variant/get.rs`, `src/transform/variant.rs`, `src/entities/variant/get/protein_change_tests.rs`, `tests/test_variant_transcript_allele_guard.py`, `CHANGELOG.md`, `sdlc/tickets/2261-accept-other-transcript-protein-changes-only-for-the-same-allele.md`, `sdlc/decisions/2026-10-19-clinical-rulings-for-the-0-9-4-review-dispositions.md`, `sdlc/tickets/2258-admit-the-0-9-4-release-candidate.md`, `sdlc/tickets/2259-check-replay-recordings-against-live-providers-before-release.md`, `sdlc/tickets/2260-share-one-replay-server-per-test-run.md`.

`get variant` accepts a protein change numbered on another transcript only when a versioned transcript projection matching the request maps to the same allele as the answer. Otherwise it refuses with the conservative MANE-compatible refusal and names the reason.

## Evidence

Clinical review on 2026-10-19 (`sdlc/decisions/2026-10-19-clinical-rulings-for-the-0-9-4-review-dispositions.md`) found a high-severity gap: `resolve_protein_change_hit` accepts a singleton hit, and `refuse_or_note_with_facts` treats a canonical residue mismatch or out-of-range position as enough proof of another transcript. Reduced tests keep only MANE SnpEff plus merged dbNSFP protein tokens and still require acceptance, which does not show a request-matching versioned transcript or the same allele.

## Keeps

Follow `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`. Keep the verified examples: `TP53 R116Q` resolving to p.Arg248Gln with its other-transcript note (ClinVar VCV000012356), and the ClinVar-free `BRCA1 Q1878R` resolving to p.Gln1857Arg, both with paired versioned projections onto one allele. No blanket refusal of non-MANE or ClinVar-free answers.

## Proof

A test with only a residue mismatch and no request-matching versioned projection refuses. The two verified examples still resolve with their notes. Each new test fails before the change. Clinical questions go to a clinical reviewer.

## Progress

- 2026-10-19 started

## Coordinator implementation grant

Require request-matching versioned transcript and paired coding/protein annotations, and a versioned paired selected-headline annotation, on the same source allele. No stem-only equivalence, residue-only inference, protein-token fallback, new provider calls, blanket ClinVar requirement or non-MANE refusal. Preserve raw source version evidence while retaining existing presentation references. Candidate singleton selection may remain if every affected acceptance path passes the shared guard. Keep both verified examples and the conservative S183Y refusal. The carried PM records and clinical decision are unchanged approved briefs; they are claimed only for landing ancestry.

## Implementation evidence

Developer stopped clean at f1d62f7509d1bdc4b091263e7fcc040d84182ca8. On exact base 86b4bb772, six authored negative CLI controls failed while two paired positives passed; the changed CLI passed all eight. Focused jobs 2261/1792432645462-15602 and 2261/1792433282697-35689 passed twenty-nine Rust cases, including conservative S183Y and absent-headline refusals, formatting, warnings-denied Clippy, Ruff and tracked-text checks. Tested source hashes match the published three-file change. No live providers or full gate ran. Existing unavailable-facts and non-substitution behavior was preserved; fresh review must assess the acceptance boundary against the approved outcome.

Keep: experiments/806-biomcp-2261 — retain controlled red/green replies, commands, tested patches and source/binary hashes.
