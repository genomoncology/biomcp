---
base: 8ab08c016d20a039c647568ca2cd285693c50c8c
head: 7529a718bd48bcc8ef93f6c13ad90f02c71b85ee
---
The hybrid ranking formula's semantic_score component is broken. `normalized_semantic_score()` reads `row.score` and clamps it to [0,1], but `row.score` is an overloaded field:

Imported from March ticket 158. The commit range was
recovered after the fact (branch march/158-fix-semantic-score-normalization-in-hybrid-ranking named for the ticket slug), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/158-fix-semantic-score-normalization-in-hybrid-ranking
