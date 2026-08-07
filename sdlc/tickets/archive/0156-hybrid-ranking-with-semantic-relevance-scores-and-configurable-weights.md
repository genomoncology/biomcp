---
flow: build
priority: 5
---
# Hybrid ranking with semantic relevance scores and configurable weights

The current article ranking uses lexical anchor matching (directness tiers) as the primary sort signal. This works for keyword-based backends but penalizes results from semantic backends like LitSense2 that match by meaning rather than by exact keyword overlap. A paper found by LitSense2 with a 0.95 semantic relevance score may rank below a paper with more keyword hits but weaker topical relevance.

Completed under March on 2026-04-09, as March ticket 156. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/156-hybrid-ranking-with-semantic-relevance-scores-and-configurable-weights
