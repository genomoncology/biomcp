---
base: 33b08fa274f790da6c36cf853abc188bcb36c674
head: 2b357f54935fedf129a707cf5db0c01b1ee302f5
---
The disease-diagnostic pivot currently relies on condition substring matching with no semantic ranking, summary-size contract, or zero- result recovery rule. That missing contract caused the tuberculosis 496 KB bloat and is not pinned by any durable architecture. The short-term cap is handled by ticket 267; this ticket formalizes the contract so future source swaps or expansions stay honest.

Imported from March ticket 275. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/275-define-disease-diagnostic-pivot-architecture-contract
