---
base: 699d4aa9addcc831da18c7d8c3108d8ccc561ed0
head: 478a40104eb3fad178aa1f39e66efd9ab714a0b6
---
When `biomcp discover` fails to resolve any biomedical entities from a query, it returns empty results with no guidance on what to try next. BioASQ evaluation (research 009) found 2+ tasks where the agent invoked discover on clinical/prevalence questions that don't map to entity resolution (e.g., "most common pediatric cerebellar tumor", "SCENAR therapy uses"), got nothing back, and then had to figure out the fallback on its own.

Imported from March ticket 204. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/204-add-article-search-fallback-suggestion-when-discover-resolves-no-entities
