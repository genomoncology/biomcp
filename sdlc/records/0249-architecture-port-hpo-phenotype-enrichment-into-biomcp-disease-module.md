---
base: b640a746696ce71801c4d665d01fa31a45e4d5c9
head: 79b0d2b8fced895b46002b9152f3e41f1edfd44f
---
Spike 243 produced a validated Python library (`clinical_features_spike`) that extracts HPO-mapped clinical features for diseases from MedlinePlus, reaching 65% expected-symptom recall with a stable output checksum and 3.6ms extraction. The spike explicitly recommends a Rust port into the BioMCP disease module rather than shelling out to the spike script. Before writing build tickets, we need architecture-level decisions: where the port lands in the Rust codebase, which new dependencies are required, whether those dependencies pass license policy, and how the work decomposes into mergeable build slices.

Imported from March ticket 249. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/249-architecture-port-hpo-phenotype-enrichment-into-biomcp-disease-module
