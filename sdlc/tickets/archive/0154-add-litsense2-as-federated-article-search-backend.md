---
flow: build
priority: 8
---
# Add LitSense2 as federated article search backend

BioMCP's federated article search currently fans out to PubTator, Europe PMC, PubMed, and Semantic Scholar — all keyword or entity-based backends. Research 011 (biomcp-article-mrr) found that 93% of gold PMIDs are unreachable by any of these backends in top-50 results. On the Hirschsprung disease task, BioMCP found 2/15 gold papers; LitSense2 sentence search found 10/15. On EWS/FLI, BioMCP found 1/7; LitSense2 passages found 6/7.

Completed under March on 2026-04-08, as March ticket 154. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/154-add-litsense2-as-federated-article-search-backend
