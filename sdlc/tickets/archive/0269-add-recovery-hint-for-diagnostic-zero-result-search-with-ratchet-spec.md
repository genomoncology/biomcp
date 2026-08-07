---
flow: build
priority: 6
---
# Add recovery hint for diagnostic zero-result search with ratchet spec

`biomcp search diagnostic --disease "nonexistent xyz"` currently outputs `No diagnostic tests found. Showing 0 of 0 results.` with no recovery suggestion. Gene and disease zero-result paths emit actionable `Try searching:` or `See also:` hints. The diagnostic zero-result path is the weakest operational path on the new surface, and no spec pins the expected recovery contract.

Completed under March on 2026-04-21, as March ticket 269. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/269-add-recovery-hint-for-diagnostic-zero-result-search-with-ratchet-spec
