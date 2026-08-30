---
flow: build
priority: 5
---

# `get pgx` misclassifies lowercase one-word drugs as genes

`biomcp get pgx warfarin annotations` returns no annotations because
`src/entities/pgx.rs::is_likely_gene` uppercases every one-word query before
checking that all characters are uppercase. This classifies lowercase drug
names such as `warfarin` and `codeine` as gene symbols and sends them through
the gene lookup path. Add an unambiguous gene-versus-drug selection rule or an
explicit mode so one-word drug queries reach the drug annotation endpoint.

## The selection rule

Promoted 2026-08-30. The draft offered a rule or a mode and settled
neither, so the choice is settled here.

`is_likely_gene` must test the query as written and must not
uppercase it first. A one-word query that is already entirely
uppercase is a gene candidate. Any other one-word query goes to the
drug path. Do not add a flag or a mode; the fix is the classifier.

## Done when

- `biomcp get pgx warfarin annotations` and the same call for
  `codeine` reach the drug annotation endpoint and return
  annotations. A test pins both.
- A one-word uppercase gene symbol still reaches the gene lookup
  path. A test pins at least one real symbol currently relied on.
- A test pins that the classifier does not uppercase its input.
- The existing suite stays green.

## Boundary

`src/entities/pgx.rs::is_likely_gene` and its tests. Do not change
the gene lookup path, the drug annotation endpoint, the CLI surface,
or multi-word query handling.
