---
base: 5dcb9e85fd6ebdfe662f15e7e08d62b6fea9c466
head: 76e74260edc82582261417c81b181776172ceb63
---
Once the per-section source-outcome contract exists, its caller-facing reporting has to be standardized and frozen. Today: hard source errors do not identify the failed source or a safe recovery action; successful CLI output is noisy while degradation is emitted only on stderr WARN — which an agent piping JSON cannot see; and there is no monotonic cross-entity ratchet, so a future source failure can silently collapse back to an empty/default in any entity.

Imported from March ticket 583. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/583-standardize-cross-entity-source-state-reporting-and-ratchet-the-contract
