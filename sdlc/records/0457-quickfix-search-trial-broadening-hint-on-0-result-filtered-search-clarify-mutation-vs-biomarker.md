---
base: fce0fd7567ac3f31f9a8bb4189b4b0f48559a16d
head: ec5b04500b0ed3f6725b98d48a8f1c98e541862c
---
`biomcp search trial` silently returns 0 results for a reasonable, well-formed query and gives the agent no way to recover, producing a false "no trials exist" conclusion.

Imported from March ticket 457. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/457-quickfix-search-trial-broadening-hint-on-0-result-filtered-search-clarify-mutation-vs-biomarker
