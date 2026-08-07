---
base: 49590ae14f66e79f91ce6b441530870e8d051f53
head: 5f42f98489b4d9f98db30b915fae216b60968d6c
---
The current article ranking uses lexical anchor matching (directness tiers) as the primary sort signal. This works for keyword-based backends but penalizes results from semantic backends like LitSense2 that match by meaning rather than by exact keyword overlap. A paper found by LitSense2 with a 0.95 semantic relevance score may rank below a paper with more keyword hits but weaker topical relevance.

Imported from March ticket 156. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/156-hybrid-ranking-with-semantic-relevance-scores-and-configurable-weights
