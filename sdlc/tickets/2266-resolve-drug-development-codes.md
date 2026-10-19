---
priority: 2
---

# 2266 — Resolve drug development codes

Status: OPEN.

Milestone: 0.9.4

## Outcome

`get drug` resolves development codes such as MK-3475 and CT-011 through ChEMBL molecule synonyms when the free-text best hit has no drug identifiers, and single and batch calls give the same answer.

## Evidence

Research ran released 0.9.3 on benchmark questions and paper fetches and verified each finding against the provider's official API; raw outputs are in research's store and may be copied as fixtures (ask the PM for the path). R5: `biomcp --json get drug MK-3475` and `get drug CT-011` return not_found, though ChEMBL lists MK-3475 as a synonym of CHEMBL3137343 and CT-011 of CHEMBL2108680; a batch call once resolved MK-3475.

## Keeps

The drug identity guards (no swapped drugs, brand and ingredient rules) stay. An ambiguous synonym refuses with candidates. Clinical identity questions go to a clinical reviewer.

## Proof

Replay tests resolve both codes in single and batch calls to the same drug; an ambiguous synonym refuses.
