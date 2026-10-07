# 2022 — Variant headline date and gene routing follow-ups

Status: OPEN.

Milestone: 0.9.2

## Outcome

The variant headline pairs a ClinVar classification with that classification's own evaluation date. Variant search routes a first token to a gene only when it is an official symbol, and a routed search that returns nothing says how it was read.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, findings 11 and 12).

- `src/transform/variant.rs:687`: `significance_evaluated` is the newest date across all ClinVar submission records, printed beside the most severe classification. `get variant 'BRAF V600E'` prints `Significance: Pathogenic — MyVariant.info (evaluated 2025-01-23)`. The 2025-01-23 record (RCV005089260) says Uncertain significance; the Pathogenic records date from 2014 to 2023. Ticket 1290 asked for the newest date, so the code follows the ticket, but the line misleads.
- `src/cli/variant/query.rs:196` accepts gene aliases. `search variant 'HCC liver cancer'` becomes `gene=HYCC1`, 0 rows, no hint. `'MODY diabetes'` routes to HNF4A only and `'HHT telangiectasia'` to ACVRL1 only. `'BRAF V600E melanoma'` becomes `gene=BRAF, condition=V600E melanoma`, 0 rows, no hint.
- The open 1291 branch adds a second newest-date helper with different rules from `newest_rcv_evaluation_date`; keep one.

- Starts from: tickets 1290 and 1301.
- Keeps: the current-classification headline and official-symbol routing.
- Changes: print the newest date among records carrying the shown classification; route on official symbols only; recognise a protein change after the gene; when a routed search returns zero rows, print the parsed form and the working alternative.
- Proof: outside-in tests for BRAF V600E's date, HCC, MODY and 'BRAF V600E melanoma', each failing on `37631c357`.
- Defers: nothing.

## Build status

- Built on branch `tickets/2022-variant-headline-date-and-gene-routing-follow-ups`,
  commits 15225975b, fe6b3ae8f, fold aa8317a4f, 2026-10-07, across two
  timeout revivals with checkpoints (nothing lost).
- Code review: REJECT 2026-10-07, findings fixed the same day. The P1
  (configuration.md still describing alias routing after the change)
  and two P2s (the contradicting enum doc; the ticket's missing
  fixture-path reason and wrong Defers) fixed in aa8317a4f, which also
  adopted the report-only hint wording (drop-or-loosen-a-filter
  instead of the misleading try-the-working-form).
- Code re-review (fold delta fe6b3ae8f..aa8317a4f): ACCEPT 2026-10-07.
  All four folds verified at the named seams with no collateral edits
  and no stale wording anywhere.
