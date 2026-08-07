---
flow: quickfix
priority: 4
---
# Move docs-integrity tests out of spec 17-cross-entity-pivots

`spec/17-cross-entity-pivots.md` mixes two unrelated concerns. It has six tests that validate documentation files and not CLI behavior: `Guide page`, `Docs navigation`, `README entry point`, `Docs home entry point`, `First query entry point`, `Quick reference entry point`. These assert that specific strings appear in `README.md`, `docs/index.md`, and related docs pages. The rest of the file tests real CLI pivot output: `Variant pivots`, `Drug to Trials`, `Disease to Drugs`, etc.

Completed under March on 2026-04-20, as March ticket 260. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/260-move-docs-integrity-tests-out-of-spec-17-cross-entity-pivots
