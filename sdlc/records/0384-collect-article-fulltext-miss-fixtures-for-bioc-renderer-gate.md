---
base: ac51a9b902b1848dc7a067979aa2e87136fe7508
head: ad1aa270bdfbd548e77e1f7b5d2a5933ecdd3445
---
Ticket 381 found that NCBI BioC was coverage-equivalent to the existing XML/HTML article fulltext ladder on the bounded sample, so BioC should not enter the default ladder without proof. The next evidence step is to collect and commit targeted fixtures where the current ladder misses or degrades and BioC materially helps, or to document that no such fixtures were found.

Imported from March ticket 384. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/384-collect-article-fulltext-miss-fixtures-for-bioc-renderer-gate
