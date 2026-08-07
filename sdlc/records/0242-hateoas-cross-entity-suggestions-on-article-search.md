---
base: ff24d38d1096eed7595fa486bc9d41220a1753fe
head: 81fae9a1f3371567aaf14727301f6dcefbef0ad3
---
When an agent searches articles with keywords that match recognizable entity patterns (drug names, disease names, gene symbols), the HATEOAS suggestions only offer more article commands (`get article`, date refinement, `--offset`). They never suggest switching to a structured command like `discover`, `get drug`, or `get disease`.

Imported from March ticket 242. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/242-hateoas-cross-entity-suggestions-on-article-search
