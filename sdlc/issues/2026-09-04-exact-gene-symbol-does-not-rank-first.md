# An exact gene symbol does not rank first in gene search

This command returned two rows on `biomcp 0.9.0-dev.6` on 2026-09-04:

```bash
biomcp --json search gene ODC1 --limit 5
```

The first row was `SLC25A21`. The exact `ODC1` symbol was second. The JSON then suggested `biomcp get gene SLC25A21` because the next-command builder used the first result.

`src/entities/gene.rs::mygene_query_term` asks MyGene for `symbol:ODC1 OR alias:ODC1`. `search_page` preserves the provider order. BioMCP already has `matching_canonical_alias_symbols`, which distinguishes a canonical-symbol match from an alias match. The response therefore contains enough information to rank the exact symbol first.

## Recommended design

Rank an exact case-insensitive symbol match before alias matches. Preserve provider order within each class. Apply the same order before pagination and before next-command generation.

This changes provider-native ordering for symbol-shaped queries. The cost is justified because the caller supplied an exact current symbol, and the current first follow-up opens a different gene.

## Done, observably

- `search gene ODC1` returns `ODC1` first.
- The first suggested detail command opens `ODC1`.
- A legacy alias still finds its canonical gene.
- Free-text gene-name searches keep provider relevance order.
- Offset pages do not repeat or omit rows after the exact-match promotion.
