---
flow: build
priority: 3
---
# Disease phenotypes: distinguish defining features from comprehensive HPO list

`get disease MONDO:... phenotypes` returns a flat HPO list with no hierarchy. All phenotypes are labeled "has phenotype" with no frequency or importance ranking. Andersen syndrome returns 13 phenotypes equally weighted. Agents dump all 13. Gold wants the 5 defining features.

Completed under March on 2026-03-29, as March ticket 081. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/081-disease-phenotypes-distinguish-defining-features-from-comprehensive-hpo-list
