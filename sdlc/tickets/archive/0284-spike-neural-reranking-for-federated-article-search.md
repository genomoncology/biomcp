---
flow: build
priority: 7
---
# Spike: neural reranking for federated article search

Research 017 (`~/workspace/research/017-biomcp-reranker/`) proved that a cross-encoder reranker applied to BioMCP's top-20 federated search candidates improves MRR from 0.179 to 0.292 — a **+63% gain** — on a 20-task BioASQ sample. The reranker understands that "hepatic steatosis" ≈ "nonalcoholic fatty liver disease" and surfaces gold papers from positions 5-12 to position 1, something the lexical anchor-counting heuristic cannot do.

Completed under March on 2026-04-22, as March ticket 284. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/284-spike-neural-reranking-for-federated-article-search

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
