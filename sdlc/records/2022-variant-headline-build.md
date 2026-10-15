# 2022 build record — variant headline date and gene routing follow-ups

Landed 2026-10-08, merge 74c0fe87f (the later d7a1323be is a
records-only follow-up, not the landing merge). Review verdicts live on
sdlc/tickets/2022-variant-headline-date-and-gene-routing-follow-ups.md.
The last review verdict before landing was REJECT with the re-review
pending; the landing did not wait for it. Ticket 2038 finding 8 orders a
fresh re-review of the landed content, which the 2038 code lane began
by fixing the dropped-filter half the REJECT named.

Outcome: the headline pairs a classification with its own evaluation
date; gene-first routing confirms official symbols only (aliases keep
the whole-phrase search with the MODY/HHT decisions recorded); a
protein change after the gene becomes its filter; a routed zero-row
search suggests dropping the condition. The landed hint code did not
keep every explicit filter at first: the routed and refused hints
dropped --significance, --tumor-site, --max-frequency, --min-cadd and
--review-status until the 2038 code lane restored them; one corner
remains, where the refused hint drops an explicit --hgvsp or
--consequence (ticket 2044 carries it). The day-shape helper stayed
duplicated at landing; the 2038 code lane already dropped the private
copy in the ClinVar path. Gates: branch CI green at 985a2317; the
completed re-reviews ACCEPT with the final round pending at landing;
merged-tree CI green on the landing tree after the receipts and
inventory unions and the duplicate-record drop.

## Landed-head re-review (2026-10-10)

- Code re-review (whole landed content at cd257afe8): FIX 2026-10-10,
  resolved by the records edits above. All four code points verified
  with evidence (the thirteen explicit filter flags in both hints, the
  one shared day rule, official-symbol-only routing with HCC, MODY and
  HHT refused, the classification-scoped evaluation date); the findings
  were this record's leftover false phrases, the wrong 2038 pointer,
  and two report-only hint corners queued under 2044.
