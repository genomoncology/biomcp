---
base: 63225990335b67959776b91dba93274d07ea03d6
head: ee0eea5d357b227a589a9943478c8a70fa7ef615
---
Operators need a scriptable way to GC orphans, evict old entries, and control cache size without wiping everything. The cache core (109) provides the snapshot/planner substrate and the cache family CLI (116) establishes the `Commands::Cache` entrypoint. This ticket adds `biomcp cache clean` — the non-destructive, targeted cleanup command — from core planner through CLI to docs and specs.

Imported from March ticket 143. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/143-cache-clean-command
