---
base: d11dc2c9b50e2a8c31616e6209c51dddf6757fed
head: c72d64f208e315009c140be413fb26ee351b429e
---
`src/entities/disease.rs` is 4,290 lines handling six concerns: type definitions, disease ID resolution/fallback logic, section enrichment handlers (genes, pathways, phenotypes, survival, funding, etc.), public search/get API, and 1,447 lines of tests. The enrichment section alone is 1,000+ lines covering 10 different data sections.

Imported from March ticket 179. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/179-decompose-disease-rs-into-disease-submodule
