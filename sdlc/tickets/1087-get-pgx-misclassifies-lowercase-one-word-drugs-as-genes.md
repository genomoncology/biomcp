---
---

# `get pgx` misclassifies lowercase one-word drugs as genes

`biomcp get pgx warfarin annotations` returns no annotations because
`src/entities/pgx.rs::is_likely_gene` uppercases every one-word query before
checking that all characters are uppercase. This classifies lowercase drug
names such as `warfarin` and `codeine` as gene symbols and sends them through
the gene lookup path. Add an unambiguous gene-versus-drug selection rule or an
explicit mode so one-word drug queries reach the drug annotation endpoint.
