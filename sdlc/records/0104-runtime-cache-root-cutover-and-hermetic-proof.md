---
base: 27373a83f50e1e1be3208811cb0d15049a2b7a64
head: 18cf79819e3974392644da183486cd9a27b19eaf
---
T101 was rejected at design-review for bundling runtime caller replacement with destructive legacy-helper cleanup in a single code step. This child ticket implements the first half: cut all live runtime callers over to `resolve_cache_config()` and add deterministic, hermetic tests for every new path contract. Legacy helpers are intentionally left in place so this change ships cleanly before the deletion sweep (T105).

Imported from March ticket 104. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/104-runtime-cache-root-cutover-and-hermetic-proof
