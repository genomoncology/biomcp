---
base: ecd7ed69d704d064f259819b870a89f3e41ccc32
head: ed58b6af46add883ab4b3cc60a3cc0316747a516
---
The standalone BioASQ benchmark harness now exists as product-owned infrastructure, but the next missing piece is the data and operations contract. Research 002 established two distinct benchmark lanes: a public mirror-derived historical corpus that is good enough for longitudinal BioMCP measurement, and an optional official BioASQ competition lane for external leaderboard claims. Right now that distinction lives only in research notes. BioMCP needs a product-owned path that can ingest the public corpus into a stable normalized schema, preserve provenance, and document what official participation would require without pretending the public mirror and the official participant downloads are the same thing.

Imported from March ticket 046. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/046-bioasq-public-corpus-and-competition-lane
