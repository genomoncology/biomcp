---
base: 95faa995d0ac86048788edc9e59ba0179da1aafb
head: 469ff2d5c8d50c59db597a732c14776ca81ec3a1
---
Vaccines have no working identity bridge in BioMCP. MyChem returns 0 for vaccine brand names (Gardasil, Comirnaty, Prevnar). WHO vaccines lack INN fields (spike 231: 57% match rate, below the 70% bar). CVX (CDC Vaccine Administered codes) is the US standard for vaccine identification — every vaccine has a CVX code, and pairing with MVX (manufacturer codes) identifies specific products. This is the foundational layer that enables all downstream vaccine work: WHO vaccines, VAERS, EMA vaccine linkage.

Imported from March ticket 233. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/233-add-cvx-vaccine-identity-layer-for-drug-entity
