---
base: 3b98894d2957a8f56963055115c6fcdc6d008990
head: 5c298b3a27104884aada9f4b43b39a5d681ebe36
---
Most BioMCP users install via pip, use it through MCP, and never think about cache management. Without automatic eviction, the cache grows unbounded until disk fills up. The cache core (109) provides config with size/disk limits, and the clean command (143) provides the orphan GC + LRU eviction logic. This ticket makes those limits enforced automatically so the default experience is safe, and surfaces cache health in `biomcp health`.

Imported from March ticket 144. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/144-automatic-cache-eviction-with-disk-aware-limits
