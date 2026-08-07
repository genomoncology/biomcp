---
flow: build
priority: 5
---
# HTTP cache directory migration helper (migration-only)

T102 was rejected at design-review for bundling the runtime HTTP cache cutover with the migration helper in a single code step. The cutover is now handled by T104 (runtime cache-root cutover and hermetic proof). This ticket implements only the second half: a non-fatal startup migration that renames `<cache_root>/http-cacache/` to `<cache_root>/http/` after T104 has established `<cache_root>/http/` as the live runtime path.

Completed under March on 2026-04-01, as March ticket 107. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/107-http-cache-directory-migration-helper-migration-only
