---
flow: build
priority: 5
---
# Define disease-diagnostic pivot architecture contract

The disease-diagnostic pivot currently relies on condition substring matching with no semantic ranking, summary-size contract, or zero- result recovery rule. That missing contract caused the tuberculosis 496 KB bloat and is not pinned by any durable architecture. The short-term cap is handled by ticket 267; this ticket formalizes the contract so future source swaps or expansions stay honest.

Completed under March on 2026-04-22, as March ticket 275. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/275-define-disease-diagnostic-pivot-architecture-contract
