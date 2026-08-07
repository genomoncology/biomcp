---
flow: build
priority: 8
---
# Wire PubMed into federated article search

BioMCP has a PubMed E-utilities backend (ticket 124) and a hydration layer (ticket 140), but PubMed is not yet exposed to users or included in default federated search. PubMed is the only backend that searches NLM's curated MeSH index, author keywords, and title/abstract fields — it finds papers that no other backend can (see notes/biomcp-article-fusion-search-vision.md for concrete failing BioASQ examples). This ticket makes PubMed a first-class article search source.

Completed under March on 2026-04-03, as March ticket 142. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/142-wire-pubmed-into-federated-article-search
