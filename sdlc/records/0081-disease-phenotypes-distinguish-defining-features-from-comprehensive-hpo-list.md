---
base: cbdb0af1769cd78efb5af1e0e26ef87e98d48020
head: 8aaa908e5e83838d52de602562cd98696a8765e1
---
`get disease MONDO:... phenotypes` returns a flat HPO list with no hierarchy. All phenotypes are labeled "has phenotype" with no frequency or importance ranking. Andersen syndrome returns 13 phenotypes equally weighted. Agents dump all 13. Gold wants the 5 defining features.

Imported from March ticket 081. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/081-disease-phenotypes-distinguish-defining-features-from-comprehensive-hpo-list
