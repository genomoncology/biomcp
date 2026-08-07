---
flow: build
priority: 5
---
# Runtime cache-root cutover and hermetic proof

T101 was rejected at design-review for bundling runtime caller replacement with destructive legacy-helper cleanup in a single code step. This child ticket implements the first half: cut all live runtime callers over to `resolve_cache_config()` and add deterministic, hermetic tests for every new path contract. Legacy helpers are intentionally left in place so this change ships cleanly before the deletion sweep (T105).

Completed under March on 2026-04-01, as March ticket 104. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/104-runtime-cache-root-cutover-and-hermetic-proof
