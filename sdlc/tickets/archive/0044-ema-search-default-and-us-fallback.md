---
flow: build
priority: 7
---
# Fix search drug --region default to all and US fallback brand-match ranking

Two bugs in `search drug` hurt the EMA integration and the existing US path:

Completed under March on 2026-03-25, as March ticket 044. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/044-ema-search-default-and-us-fallback
