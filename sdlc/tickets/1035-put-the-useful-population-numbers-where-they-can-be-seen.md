---
flow: build
priority: 12
deps: ["1022"]
---
# Put the useful population numbers where they can be seen

The gnomAD v4 population section renders roughly 130 rows. It includes every HGDP village cohort, some with sample sizes as low as a dozen alleles, and rows whose frequency renders as `-` because no alleles were observed at all. The numbers a clinician actually needs — the overall frequency in each of exomes and genomes, the ancestry group with the highest frequency, the filtering allele frequency, and the quality filter status — are correct and present, and are buried among rows that will not inform any decision.

This is a presentation problem sitting on top of a data path that was verified correct against the gnomAD API on 2026-08-19, so nothing about the underlying values should change.

The detail must remain reachable. Someone studying a specific population needs the village-level cohorts, and removing them would be a real loss. The question is what a reader sees first.

## The hard choice to settle

Decide between showing a summary by default with the full table behind a flag, and keeping the full table but ordering and grouping it so the important rows come first. The first is a better reading experience and changes what a default caller receives; the second changes nothing structurally but does less. Pick one, and say in the design how a caller who wants everything asks for it.

## Done when

- The overall exome and genome frequencies, the highest-frequency ancestry group, the filtering allele frequency, and the quality filter status are visible without scrolling past cohort rows.
- Every row available today remains reachable through a documented route.
- Rows where no alleles were observed are distinguishable from rows where the frequency is genuinely zero.
- The typed JSON form is unchanged, so anything parsing it keeps working.

## Existing tests that pin this

Restatement is authorized in `src/render/markdown/variant/tests.rs`, for these tests by name, only to the extent they assert the current row set or row order of the population table:

- `variant_markdown_renders_compact_clinvar_and_population_fields`
- `variant_population_markdown_keeps_missing_status_compact`

No other test file is authorized. Assertions about the values themselves — the grpmax FAF95 line, the caveat sentence, the quality filter status — must survive, since the data path was verified correct and is not what this ticket changes.
