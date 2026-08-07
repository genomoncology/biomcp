---
flow: build
priority: 5
---
# Fix citation count and abstract enrichment in federated article results

BioMCP's federated article search returns `citation_count: 0` and empty abstracts for papers that Semantic Scholar and PubMed have full metadata for. Research 011 found that all 50 results in a typical federated search show `citation_count=0` and `influential_citation_count=0`, even for papers with hundreds of citations in S2. Similarly, `biomcp get article <pmid>` returns no abstract for papers whose PubMed records contain full abstracts.

Completed under March on 2026-04-08, as March ticket 155. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/155-fix-citation-count-and-abstract-enrichment-in-federated-article-results
