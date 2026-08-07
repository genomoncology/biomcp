---
flow: quickfix
priority: 7
---
# Add semanticscholar as an individually selectable --source value

`semanticscholar` is a federated article source — it appears in `--source all` results and in `--debug-plan` (`source: semanticscholar`, `status: ok`) — but it is **not** an individually selectable `--source` value. The enum today is `[all, pubtator, europepmc, pubmed, litsense2]`. So a caller cannot search Semantic Scholar alone, and cannot attribute/debug it in isolation the way they can pubmed or pubtator.

Completed under March on 2026-06-17, as March ticket 421. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/421-add-semanticscholar-as-an-individually-selectable-source-value
