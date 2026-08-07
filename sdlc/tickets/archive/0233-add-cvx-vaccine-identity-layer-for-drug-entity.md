---
flow: build
priority: 7
---
# Add CVX vaccine identity layer for drug entity

Vaccines have no working identity bridge in BioMCP. MyChem returns 0 for vaccine brand names (Gardasil, Comirnaty, Prevnar). WHO vaccines lack INN fields (spike 231: 57% match rate, below the 70% bar). CVX (CDC Vaccine Administered codes) is the US standard for vaccine identification — every vaccine has a CVX code, and pairing with MVX (manufacturer codes) identifies specific products. This is the foundational layer that enables all downstream vaccine work: WHO vaccines, VAERS, EMA vaccine linkage.

Completed under March on 2026-04-17, as March ticket 233. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/233-add-cvx-vaccine-identity-layer-for-drug-entity
