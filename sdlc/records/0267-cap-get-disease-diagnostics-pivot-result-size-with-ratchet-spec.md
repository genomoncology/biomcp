---
base: 0eb1eeae53948ad147f5a80b8e45d72561ca0e6c
head: 37bbbec9f170541a0500f204f758591ba347e67b
---
`biomcp get disease tuberculosis diagnostics` produces ~496 KB of Markdown because the disease-diagnostic pivot uses substring matching against condition names — broad immunology terms match gene-centric GTR panels like "Aplastic Anemia Panel" that are only marginally related. Compounded with unbounded `Genes` cells, the output is unusable as a terminal view and is an agent-surface regression. No spec currently pins a max-size bound.

Imported from March ticket 267. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/267-cap-get-disease-diagnostics-pivot-result-size-with-ratchet-spec
