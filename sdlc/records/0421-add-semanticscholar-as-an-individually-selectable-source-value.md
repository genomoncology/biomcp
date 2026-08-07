---
base: 6f9c7d234b6b14393b7844a1eb3dc2a690809d07
head: b7dc154b8d7ae79c08ed9b16a6d8fd89c9096122
---
`semanticscholar` is a federated article source — it appears in `--source all` results and in `--debug-plan` (`source: semanticscholar`, `status: ok`) — but it is **not** an individually selectable `--source` value. The enum today is `[all, pubtator, europepmc, pubmed, litsense2]`. So a caller cannot search Semantic Scholar alone, and cannot attribute/debug it in isolation the way they can pubmed or pubtator.

Imported from March ticket 421. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/421-add-semanticscholar-as-an-individually-selectable-source-value
