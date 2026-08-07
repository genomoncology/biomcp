---
flow: build
priority: 5
---
# Keep CTGov intervention aliases literal and trial-safe

`biomcp search trial --intervention venetoclax` fails completely with a ClinicalTrials.gov HTTP 400. Alias expansion takes MyChem DrugBank synonyms presented as `brand_names`, including long IUPAC names with brackets, and sends each alias directly to CTGov's `query.intr` ESSIE parser. `--no-alias-expand` succeeds, proving the failure is introduced by BioMCP. This is a common clinical drug path and must not depend on the punctuation in upstream synonym data.

Completed under March on 2026-07-13, as March ticket 510. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/510-keep-ctgov-intervention-aliases-literal-and-trial-safe
