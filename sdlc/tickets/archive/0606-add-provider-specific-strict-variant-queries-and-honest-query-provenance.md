---
flow: build
priority: 5
---
# Add provider-specific strict variant queries and honest query provenance

BioMCP currently sends generic keyword strings to heterogeneous article providers and labels every returned row with the alias that was used to search. This creates avoidable precision loss: PubMed can translate HGVS punctuation into broad terms, Europe PMC needs provider-specific phrase and field syntax, Semantic Scholar has a bulk phrase endpoint distinct from relevance search, and PubTator query aliases are not evidence that the returned article contains the alias. The existing union must retain recall, but its provenance must distinguish searched aliases from observed evidence.

Completed under March on 2026-07-22, as March ticket 606. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/606-add-provider-specific-strict-variant-queries-and-honest-query-provenance
