---
flow: build
priority: 10
---
# Land the section-outcome failure-state proof matrix for QuickGO, STRING, InterPro, Drugs@FDA, CVX, VAERS

The typed five-state section-outcome contract (tickets 577/583) is correct at runtime — an adversarial audit (2026-07-18) traced every fetch site and confirmed a source failure completes the section as `unavailable`, and a guard prevents the success-path fallback from overwriting it with `empty`. But the deterministic native failure-state matrix is **missing** for four named sources: QuickGO (gene `go`), STRING (gene/protein `interactions`), InterPro (protein `domains`), and Drugs@FDA (drug `approvals`). Their error→unavailable classification is inline in async orchestration with no injection seam, and their test modules contain **zero** `Unavailable`/`Degraded` assertions. CVX/VAERS assert only success states (`Data`/`Empty`), never the `Unavailable` branch.

Completed under March on 2026-07-18, as March ticket 587. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/587-land-the-section-outcome-failure-state-proof-matrix-for-quickgo-string-interpro-drugs-fda-cvx-vaers
