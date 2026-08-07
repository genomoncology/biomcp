---
base: 136679850f12a0b1c69b5b7f2bbe0b4af2eef9e5
head: 252ff778ee9dafb6fd686a31c962500e86aa07c8
---
BioMCP's article search currently federates across EuropePMC (primary), PubTator3 (entity-anchored), and Semantic Scholar (enrichment). Research 009 analysis of 1,183 scored BioASQ tasks found 119 empty-answer failures where the agent searched 3–11 times and found nothing useful. Testing the same queries against PubMed E-utilities showed it found relevant results in 13/15 sampled cases where BioMCP returned nothing.

Imported from March ticket 124. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/124-add-pubmed-e-utilities-as-article-search-backend
