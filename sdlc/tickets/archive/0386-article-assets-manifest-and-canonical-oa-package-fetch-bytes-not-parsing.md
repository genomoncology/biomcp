---
flow: build
priority: 5
---
# Article assets manifest and canonical OA-package fetch (bytes not parsing)

The figures and supplementary files an agent needs already arrive inside the PMC OA `.tar.gz` package BioMCP downloads as fulltext rung 4 — but `extract_first_nxml` keeps only the `.nxml`/`.xml` entry and discards every figure image and supplementary file. So an agent that wants Supplementary Table 1 (often the paper's actual data) has no path to it even though BioMCP held the bytes. This ticket adds an on-demand asset manifest and canonical OA-package byte access, decoupled from the text waterfall. BioMCP serves bytes and a manifest; it does not parse them — turning `.xlsx`/`.doc`/`.pdf` into Markdown is downstream (Vault / machete / consumer). BioMCP's unique capability is the canonical fetch (including the OA-package ftp→https rewrite consumers cannot trivially replicate), not the extraction.

Completed under March on 2026-06-04, as March ticket 386. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/386-article-assets-manifest-and-canonical-oa-package-fetch-bytes-not-parsing
