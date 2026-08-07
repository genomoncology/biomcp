---
flow: quickfix
priority: 5
---
# Generalize get pathway identifier-redirect beyond UniProt (Ensembl, gene symbol, rsID)

436's scope said "detect an id that **matches another source's pattern** (UniProt, HGNC, etc.)" — UniProt was the *example*, not the only target. The implementation (`src/entities/pathway.rs:131-139`, `pathway_lookup_error`) checks only `crate::entities::protein::is_uniprot_accession(st_id)` and falls through to the bare error for everything else. This is scope-narrowness from 436, not a regression — a small follow-up that finishes the original intent.

Completed under March on 2026-06-24, as March ticket 443. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/443-generalize-get-pathway-identifier-redirect-beyond-uniprot-ensembl-gene-symbol-rsid
