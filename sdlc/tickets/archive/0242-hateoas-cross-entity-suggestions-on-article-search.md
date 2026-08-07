---
flow: quickfix
priority: 7
---
# HATEOAS cross-entity suggestions on article search

When an agent searches articles with keywords that match recognizable entity patterns (drug names, disease names, gene symbols), the HATEOAS suggestions only offer more article commands (`get article`, date refinement, `--offset`). They never suggest switching to a structured command like `discover`, `get drug`, or `get disease`.

Completed under March on 2026-04-18, as March ticket 242. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/242-hateoas-cross-entity-suggestions-on-article-search
