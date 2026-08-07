---
base: 6693656a47aff72cf85f27facccd9797513294ee
head: 9caabfc2c1a29575e931cbe114aa3375fd331ce2
---
Spike #449 (done) proved that connecting a variant to its 3D protein-structure context — residue, overlapping InterPro domain, PDB IDs, AlphaFold link, and Cancerhotspots recurrence — is feasible entirely within BioMCP's read-only federation using existing sources, and recommended **promote**. Build the opt-in `biomcp variant structure <variant>` helper to the contract the spike defined. This serves variant-interpretation users generally and GRIN3D-style linear-sequence-to-3D-hotspot work specifically. The spike's reference implementation and full contract live at `architecture/experiments/variant-structure-annotation/explore.md` (Outcome: promote).

Imported from March ticket 452. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/452-build-biomcp-variant-structure-variant-residue-domain-pdb-alphafold-hotspot-annotation-opt-in-no-ddg
