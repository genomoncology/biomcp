---
base: 8ab08c016d20a039c647568ca2cd285693c50c8c
head: fb56bd624c0a984ba7c76839048859556e4e5190
---
The hybrid ranking formula's semantic_score component is broken. `normalized_semantic_score()` reads `row.score` and clamps it to [0,1], but `row.score` is an overloaded field:

Imported from March ticket 158. The range was recovered after the fact, then
corrected by operator review on 2026-08-10 to the main-reachable landed commit
`fb56bd624c0a984ba7c76839048859556e4e5190`. The recorded branch range and
landed range have byte-identical ticket-owned patches after excluding discarded
`.march/**` and `target-verify158/**` artifacts; the normalized patch SHA-256 is
`99e358b571bd05e6d20cc9f15c359e932dee2aff99ffe1989465fdd641576456`.
Both commit objects exist, the recorded base is the landed head's parent, and
the landed head is an ancestor of current main.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/158-fix-semantic-score-normalization-in-hybrid-ranking
