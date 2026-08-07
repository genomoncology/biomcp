---
flow: build
priority: 6
---
# Add entity-aware HATEOAS suggestions to article search

BioASQ evaluation analysis (research 009) found that the agent's dominant failure pattern is searching articles 5-9 times with keyword reformulations when a structured command would have answered the question directly. The article search HATEOAS footer only suggests more search filters (`-k`, `-g`, `-d`, `--type`). It never suggests switching to a structured entity command even when the query clearly contains a gene symbol, drug name, or disease term.

Completed under March on 2026-04-15, as March ticket 202. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/202-add-entity-aware-hateoas-suggestions-to-article-search
