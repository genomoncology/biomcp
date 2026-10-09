# 2022 build record — variant headline date and gene routing follow-ups

Landed 2026-10-08, merge 74c0fe87f (the later d7a1323be is a
records-only follow-up, not the landing merge). Review verdicts live on
sdlc/tickets/2022-variant-headline-date-and-gene-routing-follow-ups.md.
The last review verdict before landing was REJECT with the re-review
pending; the landing did not wait for it. Ticket 2038 item 5 orders a
fresh re-review of the landed content, which ticket 2038's code lane
follows by fixing the dropped-filter half the REJECT named.

Outcome: the headline pairs a classification with its own evaluation
date; gene-first routing confirms official symbols only (aliases keep
the whole-phrase search with the MODY/HHT decisions recorded); a
protein change after the gene becomes its filter; a routed zero-row
search suggests dropping the condition with every explicit filter
kept; the day-shape helper is shared, with the collapse rule recorded
for whichever of 2022/1291 landed second (1291 landed first; its
private copy drops in a follow-up noted by ticket 2033). Gates: branch
CI green at 985a2317; the full review chain ACCEPT; merged-tree CI
green at d7a1323be after the receipts/inventory unions and the
duplicate-record drop.
