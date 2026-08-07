---
base: db7d03cb36e4aa27d4b5c352a4cef6a763de9310
head: 81b6497bf59d239ee25c7656dbe2af28ee0372c7
---
The figures and supplementary files an agent needs already arrive inside the PMC OA `.tar.gz` package BioMCP downloads as fulltext rung 4 — but `extract_first_nxml` keeps only the `.nxml`/`.xml` entry and discards every figure image and supplementary file. So an agent that wants Supplementary Table 1 (often the paper's actual data) has no path to it even though BioMCP held the bytes. This ticket adds an on-demand asset manifest and canonical OA-package byte access, decoupled from the text waterfall. BioMCP serves bytes and a manifest; it does not parse them — turning `.xlsx`/`.doc`/`.pdf` into Markdown is downstream (Vault / machete / consumer). BioMCP's unique capability is the canonical fetch (including the OA-package ftp→https rewrite consumers cannot trivially replicate), not the extraction.

Imported from March ticket 386. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/386-article-assets-manifest-and-canonical-oa-package-fetch-bytes-not-parsing
