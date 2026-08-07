---
flow: build
priority: 10
---
# Repair stale MyVariant field paths that silently return nothing

BioMCP requests a BayesDel score from MyVariant, builds a prediction entry for it, and ships nothing. The field path is stale, so the source returns no value, `push_prediction` skips the tool because both score and prediction are `None`, and the omission is silent.

Completed under March on 2026-07-15, as March ticket 541. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/541-repair-the-silently-dropped-bayesdel-predictor-and-add-the-no-af-flavor
