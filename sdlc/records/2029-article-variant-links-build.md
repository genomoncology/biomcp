# 2029 build record — article variant links prefer the gene form

Landed 2026-10-08, merge 8eec444db. Main moved at ff7d4362e
(18:00:38Z) before the merged-tree run finished; that run, 37820375572
on 8eec444db, finished green at 18:21:43Z and covers the landed tree.
An earlier record here cited 2d3815718, whose tree differs and whose
run skipped canonical-gates, windows, full-features, stress and
release-panic; that citation was wrong. Review verdicts live on the
ticket.

Outcome: a variant row carrying both a gene symbol and a protein change
prints the gene-plus-change command, so a multi-allele rsID never
decides which variant the row opens; rows without both parts keep the
earlier fallbacks. The spec states the multi-allele truth for
rs121913530 and rs121913535. Branch CI green at df760a46; the delta
through landing ACCEPTed with two P2s folded (the duplicate heading
and the report's stale count arithmetic, noted).
