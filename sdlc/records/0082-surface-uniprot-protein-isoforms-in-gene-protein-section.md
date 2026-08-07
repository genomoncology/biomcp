---
base: dfbda74a7bb48535a5b8cc32caf8345eafec0bed
head: f75f822dc3d037e58b0cdeb600e76d99acedd039
---
`get gene KRAS protein` shows the main protein function but doesn't name splice variants. UniProt has isoform data (K-Ras4A, K-Ras4B) but it's not surfaced. The gene aliases include K-RAS2A and K-RAS2B, but agents don't connect these to the human-readable isoform names.

Imported from March ticket 082. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/082-surface-uniprot-protein-isoforms-in-gene-protein-section
