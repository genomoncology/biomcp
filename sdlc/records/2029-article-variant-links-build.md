# 2029 build record — article variant links prefer the gene form

Landed 2026-10-08 (content merge 8eec444db carried to main inside the
coordinator's landing chain; merged-tree CI green at 2d3815718 on the
same tree). Review verdicts live on the ticket.

Outcome: a variant row carrying both a gene symbol and a protein change
prints the gene-plus-change command, so a multi-allele rsID never
decides which variant the row opens; rows without both parts keep the
earlier fallbacks. The spec states the multi-allele truth for
rs121913530 and rs121913535. Branch CI green at df760a46; the delta
through landing ACCEPTed with two P2s folded (the duplicate heading
and the report's stale count arithmetic, noted).
