---
base: a1e910b62a7fbea39958f9ecdb8c7f3c68285eba
head: 9b950c325cac3f13a416218006b874704c636652
---
Live GTR diagnostic search rows render unbounded joined arrays in the `Genes` and `Conditions` cells, and the `genes` JSON array contains semantic duplicates (`BRAF:B-Raf proto-oncogene...` plus bare `BRAF`). Under `--limit 3` on BRCA1 the rendered markdown table reaches 38–62 KB; under the tuberculosis disease pivot the output balloons to ~496 KB. Simple discovery commands become unscannable. No spec currently pins row compactness or gene dedupe, so the regression can recur.

Imported from March ticket 266. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/266-compact-and-dedupe-diagnostic-search-row-output-with-ratchet-spec
