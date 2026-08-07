---
base: 98d4c7743deec8eb1ac64189e340d74e0058555b
head: 7a0134a0ff4c2cc619529b6e9bf58926dd237096
---
BioMCP's cache paths are scattered across multiple callers with no single source of truth. Before any path cutover or CLI can happen, a typed config resolver must exist that handles defaults, config file values, and environment overrides in one place.

Imported from March ticket 097. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/097-canonical-cache-config-foundation
