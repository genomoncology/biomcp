---
base: d976480db6f3251f7543e5bc2aedda1390ce92df
head: 3b2af0b74b8f4ad97fd34f7ae875f96751dd8826
---
A few older consumer-facing issues remain outside the migration-ratchet cluster: JSON/list and validation-exit consistency, variant/article search precision for exact aliases, genome-build context in coordinate output, and pathway exact-title behavior. They should be reconciled with runtime behavior and pinned where still product-relevant.

Imported from March ticket 406. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/406-clarify-remaining-consumer-correctness-contracts
