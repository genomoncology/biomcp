---
flow: spike
priority: 4
---
# Collect article fulltext miss fixtures for BioC renderer gate

Ticket 381 found that NCBI BioC was coverage-equivalent to the existing XML/HTML article fulltext ladder on the bounded sample, so BioC should not enter the default ladder without proof. The next evidence step is to collect and commit targeted fixtures where the current ladder misses or degrades and BioC materially helps, or to document that no such fixtures were found.

Completed under March on 2026-05-24, as March ticket 384. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/384-collect-article-fulltext-miss-fixtures-for-bioc-renderer-gate
