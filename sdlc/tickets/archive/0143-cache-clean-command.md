---
flow: build
priority: 6
---
# Cache clean command

Operators need a scriptable way to GC orphans, evict old entries, and control cache size without wiping everything. The cache core (109) provides the snapshot/planner substrate and the cache family CLI (116) establishes the `Commands::Cache` entrypoint. This ticket adds `biomcp cache clean` — the non-destructive, targeted cleanup command — from core planner through CLI to docs and specs.

Completed under March on 2026-04-03, as March ticket 143. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/143-cache-clean-command
