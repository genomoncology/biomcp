---
flow: spike
priority: 5
---
# Spike: scope variant to protein-structure annotation (residue/domain/PDB/AlphaFold), no ddG

GRIN3D-style work needs a variant's amino-acid change tied to its 3D protein context (residue → domain → structure). BioMCP already has most of the pieces but does not connect them from a variant: - `src/sources/uniprot.rs` already extracts PDB and AlphaFold IDs and UniProt feature/domain ranges. - MyVariant.info already returns the protein position for a variant. - InterPro domains and Cancerhotspots residue recurrence are already federated.

Completed under March on 2026-06-25, as March ticket 449. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/449-spike-scope-variant-to-protein-structure-annotation-residue-domain-pdb-alphafold-no-ddg
