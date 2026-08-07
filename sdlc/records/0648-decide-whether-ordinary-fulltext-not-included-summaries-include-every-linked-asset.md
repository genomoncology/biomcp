---
base: 4ba04650f104e5eeadc83fd13b8a028cf4bac072
head: 6f29214b2f8edebbeec743b3b6942f8496632fbd
---
Ticket 600's design says attach_not_included should use the shared linked-asset resolver, but the shipped spec requires the old package-only summary; the contract needs one explicit decision.

Imported from March ticket 648. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/648-decide-whether-ordinary-fulltext-not-included-summaries-include-every-linked-asset
