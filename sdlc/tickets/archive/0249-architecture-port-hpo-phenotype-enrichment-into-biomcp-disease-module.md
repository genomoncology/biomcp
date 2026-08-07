---
flow: architect
priority: 8
---
# Architecture: port HPO phenotype enrichment into BioMCP disease module

Spike 243 produced a validated Python library (`clinical_features_spike`) that extracts HPO-mapped clinical features for diseases from MedlinePlus, reaching 65% expected-symptom recall with a stable output checksum and 3.6ms extraction. The spike explicitly recommends a Rust port into the BioMCP disease module rather than shelling out to the spike script. Before writing build tickets, we need architecture-level decisions: where the port lands in the Rust codebase, which new dependencies are required, whether those dependencies pass license policy, and how the work decomposes into mergeable build slices.

Completed under March on 2026-04-19, as March ticket 249. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/249-architecture-port-hpo-phenotype-enrichment-into-biomcp-disease-module
