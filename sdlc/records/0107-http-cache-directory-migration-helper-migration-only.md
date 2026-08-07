---
base: 71fcd9ac3eb1459db837fa0209d083d2741e5efe
head: 44334c50580c66b39c7090d6b8c3750dbade8b25
---
T102 was rejected at design-review for bundling the runtime HTTP cache cutover with the migration helper in a single code step. The cutover is now handled by T104 (runtime cache-root cutover and hermetic proof). This ticket implements only the second half: a non-fatal startup migration that renames `<cache_root>/http-cacache/` to `<cache_root>/http/` after T104 has established `<cache_root>/http/` as the live runtime path.

Imported from March ticket 107. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/107-http-cache-directory-migration-helper-migration-only
