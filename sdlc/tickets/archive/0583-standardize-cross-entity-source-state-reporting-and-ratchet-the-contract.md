---
flow: build
priority: 10
---
# Standardize cross-entity source-state reporting and ratchet the contract

Once the per-section source-outcome contract exists, its caller-facing reporting has to be standardized and frozen. Today: hard source errors do not identify the failed source or a safe recovery action; successful CLI output is noisy while degradation is emitted only on stderr WARN — which an agent piping JSON cannot see; and there is no monotonic cross-entity ratchet, so a future source failure can silently collapse back to an empty/default in any entity.

Completed under March on 2026-07-17, as March ticket 583. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/583-standardize-cross-entity-source-state-reporting-and-ratchet-the-contract
