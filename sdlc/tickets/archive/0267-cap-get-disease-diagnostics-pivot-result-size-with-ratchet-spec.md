---
flow: build
priority: 8
---
# Cap get disease diagnostics pivot result size with ratchet spec

`biomcp get disease tuberculosis diagnostics` produces ~496 KB of Markdown because the disease-diagnostic pivot uses substring matching against condition names — broad immunology terms match gene-centric GTR panels like "Aplastic Anemia Panel" that are only marginally related. Compounded with unbounded `Genes` cells, the output is unusable as a terminal view and is an agent-surface regression. No spec currently pins a max-size bound.

Completed under March on 2026-04-21, as March ticket 267. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/267-cap-get-disease-diagnostics-pivot-result-size-with-ratchet-spec
