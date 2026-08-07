---
flow: build
priority: 9
---
# Fix PubMed E-utilities query construction AND NOT bug

Every PubMed search in BioMCP silently returns 0 results because `build_pubmed_search_term` (src/entities/article.rs:1407) joins all clauses with `AND`, producing queries like `WDR5 AND NOT retracted publication[pt]`. PubMed's E-utilities cannot parse `AND NOT` — it needs standalone `NOT` as a boolean operator. The API mangles the query into `WDR5 AND "retracted publication"[Publication Type]`, which matches nothing.

Completed under March on 2026-04-04, as March ticket 145. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/145-fix-pubmed-e-utilities-query-construction-and-not-bug
