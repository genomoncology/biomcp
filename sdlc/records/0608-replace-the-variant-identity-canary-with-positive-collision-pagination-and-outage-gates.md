---
base: dfaebf578d60f1b3ddc7f89365f41655ab5b03d6
head: 977887f01e75347ba9f5570763d7f5f833d6c832
---
The current seven-variant canary proves that a query alias was attached to a result and that an exact route ran. It does not prove that returned content contains the requested gene and allele, that collisions are rejected, that verification happens before pagination, or that provider outages are honestly reported. This leaves the release gate unable to detect the failure mode it claims to protect against.

Imported from March ticket 608. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/608-replace-the-variant-identity-canary-with-positive-collision-pagination-and-outage-gates
