---
base: cab97ae10a18be7fb9ac3c702bb8f520f1048d04
head: 5b20afdb473949e302a2bef4d1a08e5a9150cb9c
---
BioMCP's `get disease <id> phenotypes` returns HPO (Human Phenotype Ontology) annotations from OMIM and HPO-annotations sources. For many common diseases, these annotations are sparse or focus on histological/genetic features rather than clinical symptoms patients and clinicians recognize.

Imported from March ticket 243. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/243-architecture-hpo-phenotype-enrichment-for-clinical-symptoms
