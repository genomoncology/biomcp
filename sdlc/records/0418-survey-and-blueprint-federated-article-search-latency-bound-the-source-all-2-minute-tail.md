---
base: 75f009fd1fbf434ed19f9c3cdba0dd47635e922f
head: 6ee0986394db6946bcb292bf4c5aac7730266778
---
The default federated `biomcp search article` (`--source all`) takes ~135s wall, versus ~13-15s for a single source (`--source pubmed` / `--source pubtator`) on the same machine — roughly 10x slower. The fan-out federates PubTator3, Europe PMC, PubMed, LitSense2, and Semantic Scholar plus a cross-source dedup/ranking (merge) step; fewer keywords on `--source all` was still ~137s, so the cost is the federation, not the query.

Imported from March ticket 418. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/418-survey-and-blueprint-federated-article-search-latency-bound-the-source-all-2-minute-tail
