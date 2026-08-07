---
flow: build
priority: 7
---
# Make Semantic Scholar work without S2_API_KEY

Semantic Scholar's API works without authentication on a shared rate-limit pool. BioMCP currently hard-gates all S2 features behind `S2_API_KEY`, which blocks users who don't qualify for a key (S2 prioritizes academic institutions). GitHub issue #225 reports this with a working curl proof.

Completed under March on 2026-03-25, as March ticket 047. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/047-s2-optional-api-key
