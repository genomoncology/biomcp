---
base: dd70e889aa4015b3abec94be15acbc787c29a4c4
head: d3cdf559351392b103ef4099ac107afd238495dd
---

The execve-traced full gate exhausted eight-second test-lane deadlines while local OLS4, dbSNP, and gnomAD fixtures were still making progress. Tracing also exposed undersized subprocess budgets in the output-footprint and variant-normalization contracts.

Test execution now uses 300-second OLS4 and population-enrichment budgets while production deadlines remain unchanged. The two affected Python contracts use the same tracing-tolerant subprocess budget, and the Rust source-size inventory accounts for the focused timeout selection.

Validation passed with `sh sdlc/scripts/lint`, `sh sdlc/scripts/test`, and the full test gate under execve tracing.
