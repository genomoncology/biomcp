---
base: 4d4ae0628ede194daaffa564b6585ee9f2ce6583
head: 3ac10ffa80227f8fcb0d80fe2b7339b1296f86cc
---
Survey issues 1, 2, 3, and 7 show that BioMCP lacks a shared planning boundary for rare-disease trial workflows. `discover`, `search trial`, `gene trials`, and `disease trials` each route directly to command strings or flat `TrialSearchFilters`, so there is no deterministic place to represent disease labels, gene handles, bounded condition expansions, provenance, or noisy-expansion rejections.

Imported from March ticket 414. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/414-add-rare-disease-trial-planning-request-contracts
