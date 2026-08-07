---
base: e05ef4b8d8489c2ccf01327de3a8aa4fda05c414
head: 14e10b251d489c21c7bb2ef4f74b257b2b7378ea
---
The typed five-state section-outcome contract (tickets 577/583) is correct at runtime — an adversarial audit (2026-07-18) traced every fetch site and confirmed a source failure completes the section as `unavailable`, and a guard prevents the success-path fallback from overwriting it with `empty`. But the deterministic native failure-state matrix is **missing** for four named sources: QuickGO (gene `go`), STRING (gene/protein `interactions`), InterPro (protein `domains`), and Drugs@FDA (drug `approvals`). Their error→unavailable classification is inline in async orchestration with no injection seam, and their test modules contain **zero** `Unavailable`/`Degraded` assertions. CVX/VAERS assert only success states (`Data`/`Empty`), never the `Unavailable` branch.

Imported from March ticket 587. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/587-land-the-section-outcome-failure-state-proof-matrix-for-quickgo-string-interpro-drugs-fda-cvx-vaers
