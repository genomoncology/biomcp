---
priority: 2
---

# 2181 — Search full text in the exact variant name strategy

Status: OPEN.

Milestone: 0.9.3

## Outcome

The exact variant-name strategy searches full text wherever a source supports it, so a variant named only in a paper's body or tables is found.

- Evidence: research's spot check: `--strategy exact` for `BRCA1 F1761I` finds 0 papers while the union strategy's lexical route finds 10811118 under the same name. `src/entities/article/variant_search.rs:3085-3094` wraps the name in PubMed `[Title/Abstract]` and Europe PMC `TITLE_ABS:` only.
- Keeps: one exact name per query, per-alias and per-source yield in `alias_yields`, and PubMed's title and abstract phrase search, since PubMed has no full-text field.
- Changes: claims `src/entities/article/variant_search.rs`, its tests and recorded replies, `CHANGELOG.md`. Search Europe PMC by the exact phrase across full text, not `TITLE_ABS`, and report it under its own query key.
- Proof: recorded replies show `BRCA1 F1761I` finding 10811118 through the exact strategy; current main finds none. Alias yields stay per source.
