---
base: a3ea6a317565068f04c89d67f41f661d12d0426c
head: 0ca01c054510dd65d3f736421b8cc979eb441d51
---
436's scope said "detect an id that **matches another source's pattern** (UniProt, HGNC, etc.)" — UniProt was the *example*, not the only target. The implementation (`src/entities/pathway.rs:131-139`, `pathway_lookup_error`) checks only `crate::entities::protein::is_uniprot_accession(st_id)` and falls through to the bare error for everything else. This is scope-narrowness from 436, not a regression — a small follow-up that finishes the original intent.

Imported from March ticket 443. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/443-generalize-get-pathway-identifier-redirect-beyond-uniprot-ensembl-gene-symbol-rsid
