---
base: 85cea35cb8ad79adf942c24c1a9252ccdb895fa3
head: c8785f4bdedcc46823751372eb8e43acaf3e4c17
---
Current BioMCP variant lookup supports rsID, genomic HGVS-like `chrN:g.posRef>Alt`, and gene+protein shorthand through MyVariant-backed surfaces, but transcript HGVS such as `NM_000248.3:c.135del` is rejected. Public services like Mutalyzer and VariantValidator can validate those source-shaped strings, return normalized source output, and report transcript/version warnings without BioMCP inventing interpretation logic.

Imported from March ticket 370. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/370-add-transcript-hgvs-normalization-proxies
