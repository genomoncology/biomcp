---
flow: build
priority: 9
---
# Add deterministic article identity verification with auditable observations

BioMCP currently treats the alias used in a search as though it were observed in the article. That makes provider retrieval collisions appear to be exact variant matches. BioMCP owns provider responses, captured content, provenance, caching, and CLI/MCP parity, so it should add a deterministic identity-verification module without interpreting experiments or making ACMG decisions.

Completed under March on 2026-07-22, as March ticket 607. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/607-add-deterministic-article-identity-verification-with-auditable-observations
