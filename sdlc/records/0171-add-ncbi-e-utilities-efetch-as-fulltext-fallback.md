---
base: 93ae34b89df68ddea469d90924425574c114776c
head: eba51610b6b36c1689224579bd91864eb7e17cea
---
Article fulltext retrieval is completely broken — 0 of 6 tested PMC articles returned full text. This blocks any workflow that depends on reading papers (literature synthesis, evidence extraction, cancer survival data analysis). NCBI E-utilities efetch is a reliable, free, no-auth-required alternative that works for all tested articles.

Imported from March ticket 171. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/171-add-ncbi-e-utilities-efetch-as-fulltext-fallback
