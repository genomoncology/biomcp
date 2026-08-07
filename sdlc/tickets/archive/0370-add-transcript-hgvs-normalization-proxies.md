---
flow: build
priority: 8
---
# Add transcript HGVS normalization proxies

Current BioMCP variant lookup supports rsID, genomic HGVS-like `chrN:g.posRef>Alt`, and gene+protein shorthand through MyVariant-backed surfaces, but transcript HGVS such as `NM_000248.3:c.135del` is rejected. Public services like Mutalyzer and VariantValidator can validate those source-shaped strings, return normalized source output, and report transcript/version warnings without BioMCP inventing interpretation logic.

Completed under March on 2026-05-22, as March ticket 370. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/370-add-transcript-hgvs-normalization-proxies
