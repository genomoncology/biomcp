---
base: 5a748d943882d9a0a813b076a03fcfc653891cfc
head: e7fb6cc76ebe8296dbc431ce0cf05319f35b4505
---
GRIN3D-style work needs a variant's amino-acid change tied to its 3D protein context (residue → domain → structure). BioMCP already has most of the pieces but does not connect them from a variant: - `src/sources/uniprot.rs` already extracts PDB and AlphaFold IDs and UniProt feature/domain ranges. - MyVariant.info already returns the protein position for a variant. - InterPro domains and Cancerhotspots residue recurrence are already federated.

Imported from March ticket 449. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/449-spike-scope-variant-to-protein-structure-annotation-residue-domain-pdb-alphafold-no-ddg
