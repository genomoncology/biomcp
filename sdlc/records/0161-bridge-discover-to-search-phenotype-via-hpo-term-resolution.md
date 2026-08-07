---
base: f68a2589043cd3b97cf825b60f524548751d21b7
head: 8b6f9304bb461d612cfd2a46b711d6b318ddcf6e
---
`discover` queries OLS4 which already searches the HPO ontology, but when HPO terms are found they are not surfaced prominently and `search phenotype` is never suggested as a follow-up command. This means an agent that starts from free-text symptoms ("febrile seizures, developmental delay") cannot reach the phenotype-to-disease matching pipeline without already knowing HPO codes.

Imported from March ticket 161. The commit range was
recovered after the fact (branch march/161-bridge-discover-to-search-phenotype-via-hpo-term-resolution named for the ticket slug), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/161-bridge-discover-to-search-phenotype-via-hpo-term-resolution
