---
flow: build
priority: 5
---
# Bridge discover to search phenotype via HPO term resolution

`discover` queries OLS4 which already searches the HPO ontology, but when HPO terms are found they are not surfaced prominently and `search phenotype` is never suggested as a follow-up command. This means an agent that starts from free-text symptoms ("febrile seizures, developmental delay") cannot reach the phenotype-to-disease matching pipeline without already knowing HPO codes.

Completed under March on 2026-04-10, as March ticket 161. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/161-bridge-discover-to-search-phenotype-via-hpo-term-resolution
