---
flow: build
priority: 5
---
# Cache core: config origins and reusable snapshot/planner

Tickets 095B, 095C, and 095D (cache CLI commands) all need access to the same cache inspection and cleanup planning logic, but that logic doesn't exist as a reusable internal module yet. This ticket adds the internal cache-core foundation that downstream CLI tickets call — no new user-visible commands.

Completed under March on 2026-04-02, as March ticket 109. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/109-cache-core-config-origins-and-reusable-snapshot-planner
