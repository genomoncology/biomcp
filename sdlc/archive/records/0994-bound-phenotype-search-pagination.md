---
flow: quickfix
priority: 10
---

# Bound phenotype search pagination

Phenotype search must never claim that an empty page beyond Monarch's first 50 results proves completion. Accept at most ten unique HPO terms, reject a requested window beyond 50 before Monarch contact, expose no invented total, and distinguish a followable next page from possible provider truncation.

Red-green coverage belongs in `src/entities/disease/search/tests.rs`, `src/cli/phenotype/tests.rs`, and `spec/entity/phenotype.md`; their pagination assertions may be restated.
