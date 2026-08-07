---
base: 9dcdec0a604ced0caf5649c90fb333aabef1ecff
head: b9b2641678def057f429ff07a0f02f6e52ab5a95
---
BioMCP has a PubMed E-utilities backend (ticket 124) and a hydration layer (ticket 140), but PubMed is not yet exposed to users or included in default federated search. PubMed is the only backend that searches NLM's curated MeSH index, author keywords, and title/abstract fields — it finds papers that no other backend can (see notes/biomcp-article-fusion-search-vision.md for concrete failing BioASQ examples). This ticket makes PubMed a first-class article search source.

Imported from March ticket 142. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/142-wire-pubmed-into-federated-article-search
