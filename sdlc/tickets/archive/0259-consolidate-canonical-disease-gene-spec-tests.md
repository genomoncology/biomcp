---
flow: quickfix
priority: 4
---
# Consolidate canonical disease gene spec tests

`spec/07-disease.md` contains four near-duplicate tests that each call `biomcp get disease <name>` with a different canonical disease and assert a specific gene list: `Canonical CLL Disease Genes`, `Canonical T-PLL Disease Genes`, `Canonical Parkinson Disease Genes`, `Canonical CMT1A Disease Genes`. Each hits OpenTargets with a distinct MONDO ID, so each is a separate cold-cache network request. They duplicate the same CLI contract — "canonical disease card surfaces a disease-genes table" — four times at four different wall-time costs.

Completed under March on 2026-04-20, as March ticket 259. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/259-consolidate-canonical-disease-gene-spec-tests
