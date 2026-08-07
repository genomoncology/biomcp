---
base: 7ebb965bdae8ed23eac9eb3c04ce3d3813f415c1
head: 71531ab64aba42aeae4e6cd05bec4eb18394a2dd
---
`get disease <name> clinical_features` does not proxy a backend — it serves a **hand-curated answer key for exactly 3 diseases** (uterine fibroid, endometriosis, chronic venous insufficiency), dressed up as MedlinePlus-sourced data, and falls through to "no curated entry" for everything else. This is the clearest proxy-purity violation in the codebase.

Imported from March ticket 493. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/493-rip-out-clinical-features-curated-answer-key-reroute-directly-to-monarch-hpo-backend-no-compat-projection
