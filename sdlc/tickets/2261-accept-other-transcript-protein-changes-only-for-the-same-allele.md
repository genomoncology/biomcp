---
priority: 1
---

# 2261 — Accept other-transcript protein changes only for the same allele

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get variant` accepts a protein change numbered on another transcript only when a versioned transcript projection matching the request maps to the same allele as the answer. Otherwise it refuses with the conservative MANE-compatible refusal and names the reason.

## Evidence

Clinical review on 2026-10-19 (`sdlc/decisions/2026-10-19-clinical-rulings-for-the-0-9-4-review-dispositions.md`) found a high-severity gap: `resolve_protein_change_hit` accepts a singleton hit, and `refuse_or_note_with_facts` treats a canonical residue mismatch or out-of-range position as enough proof of another transcript. Reduced tests keep only MANE SnpEff plus merged dbNSFP protein tokens and still require acceptance, which does not show a request-matching versioned transcript or the same allele.

## Keeps

Follow `sdlc/decisions/2026-10-18-transcript-aliases-need-the-same-genomic-change.md`. Keep the verified examples: `TP53 R116Q` resolving to p.Arg248Gln with its other-transcript note (ClinVar VCV000012356), and the ClinVar-free `BRCA1 Q1878R` resolving to p.Gln1857Arg, both with paired versioned projections onto one allele. No blanket refusal of non-MANE or ClinVar-free answers.

## Proof

A test with only a residue mismatch and no request-matching versioned projection refuses. The two verified examples still resolve with their notes. Each new test fails before the change. Clinical questions go to a clinical reviewer.

## Progress

- 2026-10-19 started
