---
base: 9b7f1aeab47ccd6e25cb5690f1808c6a855f54af
head: 95c22b71eb188a0cbe59b78c5a830de6483458f9
---
`src/entities/variant.rs` is 2,534 lines — a hot surface handling variant search, detail retrieval, population frequencies, and ClinVar/CIViC enrichment. After article/drug/disease/trial decompositions, variant is the next remaining god-file on the entity tier. Shrink it into `src/entities/variant/` following the established submodule pattern.

Imported from March ticket 190. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/190-decompose-variant-rs-into-variant-submodule
