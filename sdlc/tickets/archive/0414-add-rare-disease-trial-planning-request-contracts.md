---
flow: build
priority: 9
---
# Add rare-disease trial planning request contracts

Survey issues 1, 2, 3, and 7 show that BioMCP lacks a shared planning boundary for rare-disease trial workflows. `discover`, `search trial`, `gene trials`, and `disease trials` each route directly to command strings or flat `TrialSearchFilters`, so there is no deterministic place to represent disease labels, gene handles, bounded condition expansions, provenance, or noisy-expansion rejections.

Completed under March on 2026-06-14, as March ticket 414. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/414-add-rare-disease-trial-planning-request-contracts
