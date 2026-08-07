---
base: 4bef72e1ba0ed3c74dd979f901ae7a4736a7d54b
head: e595ed2d335b283264bb6e6f2cf229276c50a99e
---
Semantic Scholar's API works without authentication on a shared rate-limit pool. BioMCP currently hard-gates all S2 features behind `S2_API_KEY`, which blocks users who don't qualify for a key (S2 prioritizes academic institutions). GitHub issue #225 reports this with a working curl proof.

Imported from March ticket 047. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/047-s2-optional-api-key
