---
flow: quickfix
priority: 7
---
# Add OLS4 disease/discover parallel-isolation contract to absorb FAQ #14

FAQ watching entry #14 says OLS4 discover fallbacks are slow enough to flake under `-n auto --dist loadfile` and should be absorbed by serial or fixture-backed lanes. The Makefile only carves out `spec/entity/protein.md` today. The 327 code review found that `spec/entity/disease.md` and `spec/surface/discover.md` still call OLS4-backed discover headings (`Resolved via discover + crosswalk`, synonym rescue, alias routing, symptom mapping), and there is no automated check that prevents new OLS4-heavy headings from being added back into the parallel pool.

Completed under March on 2026-04-28, as March ticket 336. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/336-add-ols4-disease-discover-parallel-isolation-contract-to-absorb-faq-14
