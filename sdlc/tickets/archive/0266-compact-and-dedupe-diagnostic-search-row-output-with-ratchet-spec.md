---
flow: build
priority: 8
---
# Compact and dedupe diagnostic search row output with ratchet spec

Live GTR diagnostic search rows render unbounded joined arrays in the `Genes` and `Conditions` cells, and the `genes` JSON array contains semantic duplicates (`BRAF:B-Raf proto-oncogene...` plus bare `BRAF`). Under `--limit 3` on BRCA1 the rendered markdown table reaches 38–62 KB; under the tuberculosis disease pivot the output balloons to ~496 KB. Simple discovery commands become unscannable. No spec currently pins row compactness or gene dedupe, so the regression can recur.

Completed under March on 2026-04-21, as March ticket 266. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/266-compact-and-dedupe-diagnostic-search-row-output-with-ratchet-spec
