---
base: 9b92421a81fa477c5a4aaf38683444309628a69d
head: 049a665d35e234dfe34a0fc89e0ff29af458fb72
---
Every PubMed search in BioMCP silently returns 0 results because `build_pubmed_search_term` (src/entities/article.rs:1407) joins all clauses with `AND`, producing queries like `WDR5 AND NOT retracted publication[pt]`. PubMed's E-utilities cannot parse `AND NOT` — it needs standalone `NOT` as a boolean operator. The API mangles the query into `WDR5 AND "retracted publication"[Publication Type]`, which matches nothing.

Imported from March ticket 145. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/145-fix-pubmed-e-utilities-query-construction-and-not-bug
