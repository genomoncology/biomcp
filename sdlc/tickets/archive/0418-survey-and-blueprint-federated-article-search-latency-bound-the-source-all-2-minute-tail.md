---
flow: build
priority: 10
---
# Bound federated article-search latency (--source all): per-source timeout + concurrent fan-out with honest degradation

The default federated `biomcp search article` (`--source all`) takes ~135s wall, versus ~13-15s for a single source (`--source pubmed` / `--source pubtator`) on the same machine — roughly 10x slower. The fan-out federates PubTator3, Europe PMC, PubMed, LitSense2, and Semantic Scholar plus a cross-source dedup/ranking (merge) step; fewer keywords on `--source all` was still ~137s, so the cost is the federation, not the query.

Completed under March on 2026-06-14, as March ticket 418. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/418-survey-and-blueprint-federated-article-search-latency-bound-the-source-all-2-minute-tail
