---
flow: architect
priority: 10
---
# Design spec-v2 corpus: 13 entity + 3 surface specs, biomcp-ci wrapper, cache-warm gate

The biomcp spec corpus is in structural rot: 27 files, ~8,456 lines, ~1,700 mustmatch assertions, fanning out to ~10 live upstream APIs per PR run, with exact-count and exact-prose trivia pinning that treats copy edits as regressions. The five open tickets (287, 293, 294, 295, 296) are all bandaids on one underlying pain chain:

Completed under March on 2026-04-24, as March ticket 297. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/297-design-spec-v2-corpus-13-entity-3-surface-specs-biomcp-ci-wrapper-cache-warm-gate
