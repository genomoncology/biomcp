---
base: 0dbb7a0d98c50bf781b5c5aa18d8b3521395167f
head: 036b36dd1b507fe1bc24b7d5ae94a851f66d02c8
---
Prefer the canonical HGNC gene for a known gene alias over NCIT/MESH drug/symptom entities (PD-L1 returns drugs, omits CD274); fold in HER2/P53 hint + alias-set audit. Verified high-severity data bug on 0.8.24.

Imported from March ticket 439. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/439-alias-resolver-prefer-canonical-hgnc-gene-over-ncit-mesh-for-known-gene-aliases-pd-l1-her2-p53
