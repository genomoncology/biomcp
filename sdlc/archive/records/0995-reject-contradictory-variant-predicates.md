---
flow: quickfix
priority: 10
---

# Reject contradictory variant predicates

Variant search must reject a field marked missing when another supplied filter requires that same field. This includes direct `--has`/`--missing` conflicts and the CADD, REVEL, GERP, gnomAD, ClinVar, SnpEff, CIViC, and COSMIC filter families. Rejection happens before provider contact and is shared by CLI and MCP execution.

Red-green coverage belongs in `src/sources/myvariant/tests/construction.rs`, `src/entities/variant/search/tests.rs`, and the existing CLI/MCP variant process contracts; their former provider-contact expectations may be restated.
