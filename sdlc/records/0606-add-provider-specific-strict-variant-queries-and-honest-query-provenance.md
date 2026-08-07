---
base: ecf950af3c43f3ae2418a6ee26c470c2f3683ac9
head: 400d6ea542d3b529ba1386b24301da53a274a497
---
BioMCP currently sends generic keyword strings to heterogeneous article providers and labels every returned row with the alias that was used to search. This creates avoidable precision loss: PubMed can translate HGVS punctuation into broad terms, Europe PMC needs provider-specific phrase and field syntax, Semantic Scholar has a bulk phrase endpoint distinct from relevance search, and PubTator query aliases are not evidence that the returned article contains the alias. The existing union must retain recall, but its provenance must distinguish searched aliases from observed evidence.

Imported from March ticket 606. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/606-add-provider-specific-strict-variant-queries-and-honest-query-provenance
