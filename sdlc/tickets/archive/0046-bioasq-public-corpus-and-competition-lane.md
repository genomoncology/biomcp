---
flow: build
priority: 6
---
# Extend BioASQ benchmark module with public corpus ingestion and official-competition runbook

The standalone BioASQ benchmark harness now exists as product-owned infrastructure, but the next missing piece is the data and operations contract. Research 002 established two distinct benchmark lanes: a public mirror-derived historical corpus that is good enough for longitudinal BioMCP measurement, and an optional official BioASQ competition lane for external leaderboard claims. Right now that distinction lives only in research notes. BioMCP needs a product-owned path that can ingest the public corpus into a stable normalized schema, preserve provenance, and document what official participation would require without pretending the public mirror and the official participant downloads are the same thing.

Completed under March on 2026-03-25, as March ticket 046. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/046-bioasq-public-corpus-and-competition-lane
