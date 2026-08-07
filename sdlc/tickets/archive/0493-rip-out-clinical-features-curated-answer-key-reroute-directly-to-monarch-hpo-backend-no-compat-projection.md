---
flow: build
priority: 5
---
# Rip out clinical_features curated answer key, reroute directly to Monarch/HPO backend (no compat projection)

`get disease <name> clinical_features` does not proxy a backend — it serves a **hand-curated answer key for exactly 3 diseases** (uterine fibroid, endometriosis, chronic venous insufficiency), dressed up as MedlinePlus-sourced data, and falls through to "no curated entry" for everything else. This is the clearest proxy-purity violation in the codebase.

Completed under March on 2026-07-09, as March ticket 493. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/493-rip-out-clinical-features-curated-answer-key-reroute-directly-to-monarch-hpo-backend-no-compat-projection
