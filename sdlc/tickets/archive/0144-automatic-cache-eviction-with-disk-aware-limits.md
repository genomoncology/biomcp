---
flow: build
priority: 4
---
# Automatic cache eviction with disk-aware limits

Most BioMCP users install via pip, use it through MCP, and never think about cache management. Without automatic eviction, the cache grows unbounded until disk fills up. The cache core (109) provides config with size/disk limits, and the clean command (143) provides the orphan GC + LRU eviction logic. This ticket makes those limits enforced automatically so the default experience is safe, and surfaces cache health in `biomcp health`.

Completed under March on 2026-04-04, as March ticket 144. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/144-automatic-cache-eviction-with-disk-aware-limits
