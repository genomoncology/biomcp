---
flow: build
priority: 5
---
# Fix semantic_score normalization in hybrid ranking

The hybrid ranking formula's semantic_score component is broken. `normalized_semantic_score()` reads `row.score` and clamps it to [0,1], but `row.score` is an overloaded field:

Completed under March on 2026-04-09, as March ticket 158. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/158-fix-semantic-score-normalization-in-hybrid-ranking
