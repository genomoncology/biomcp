---
flow: build
priority: 9
---
# Add PubMed E-utilities as article search backend

BioMCP's article search currently federates across EuropePMC (primary), PubTator3 (entity-anchored), and Semantic Scholar (enrichment). Research 009 analysis of 1,183 scored BioASQ tasks found 119 empty-answer failures where the agent searched 3–11 times and found nothing useful. Testing the same queries against PubMed E-utilities showed it found relevant results in 13/15 sampled cases where BioMCP returned nothing.

Completed under March on 2026-04-03, as March ticket 124. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/124-add-pubmed-e-utilities-as-article-search-backend
