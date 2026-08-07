---
flow: build
priority: 6
---
# Canonical cache config foundation

BioMCP's cache paths are scattered across multiple callers with no single source of truth. Before any path cutover or CLI can happen, a typed config resolver must exist that handles defaults, config file values, and environment overrides in one place.

Completed under March on 2026-04-01, as March ticket 097. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/097-canonical-cache-config-foundation
