---
flow: build
priority: 5
---
# Build: biomcp variant structure <variant> — residue/domain/PDB/AlphaFold/hotspot annotation (opt-in, no ddG)

Spike #449 (done) proved that connecting a variant to its 3D protein-structure context — residue, overlapping InterPro domain, PDB IDs, AlphaFold link, and Cancerhotspots recurrence — is feasible entirely within BioMCP's read-only federation using existing sources, and recommended **promote**. Build the opt-in `biomcp variant structure <variant>` helper to the contract the spike defined. This serves variant-interpretation users generally and GRIN3D-style linear-sequence-to-3D-hotspot work specifically. The spike's reference implementation and full contract live at `architecture/experiments/variant-structure-annotation/explore.md` (Outcome: promote).

Completed under March on 2026-06-25, as March ticket 452. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/452-build-biomcp-variant-structure-variant-residue-domain-pdb-alphafold-hotspot-annotation-opt-in-no-ddg
