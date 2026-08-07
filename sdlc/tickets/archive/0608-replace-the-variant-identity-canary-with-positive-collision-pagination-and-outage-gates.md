---
flow: build
priority: 9
---
# Replace the variant identity canary with positive collision pagination and outage gates

The current seven-variant canary proves that a query alias was attached to a result and that an exact route ran. It does not prove that returned content contains the requested gene and allele, that collisions are rejected, that verification happens before pagination, or that provider outages are honestly reported. This leaves the release gate unable to detect the failure mode it claims to protect against.

Completed under March on 2026-07-22, as March ticket 608. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/608-replace-the-variant-identity-canary-with-positive-collision-pagination-and-outage-gates
