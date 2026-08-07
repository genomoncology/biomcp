---
base: d85cfeb26e7d8634857e33b88a98c1bfbf222693
head: bf338c9a09ed78980f0ce1005abb6fe26f492e87
---
BioMCP's federated article search returns `citation_count: 0` and empty abstracts for papers that Semantic Scholar and PubMed have full metadata for. Research 011 found that all 50 results in a typical federated search show `citation_count=0` and `influential_citation_count=0`, even for papers with hundreds of citations in S2. Similarly, `biomcp get article <pmid>` returns no abstract for papers whose PubMed records contain full abstracts.

Imported from March ticket 155. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/155-fix-citation-count-and-abstract-enrichment-in-federated-article-results
