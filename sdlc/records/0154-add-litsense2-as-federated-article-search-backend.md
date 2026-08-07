---
base: daedeb41b20f9904610c082963917d5381a62489
head: 249d277db6bef2af4709b62acb285fd8f610e423
---
BioMCP's federated article search currently fans out to PubTator, Europe PMC, PubMed, and Semantic Scholar — all keyword or entity-based backends. Research 011 (biomcp-article-mrr) found that 93% of gold PMIDs are unreachable by any of these backends in top-50 results. On the Hirschsprung disease task, BioMCP found 2/15 gold papers; LitSense2 sentence search found 10/15. On EWS/FLI, BioMCP found 1/7; LitSense2 passages found 6/7.

Imported from March ticket 154. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/154-add-litsense2-as-federated-article-search-backend
