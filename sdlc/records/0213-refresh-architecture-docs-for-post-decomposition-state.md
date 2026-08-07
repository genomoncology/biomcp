---
base: 1b76c0570d15b75a1abc9eabc168aa35062fc149
head: 9159be7089140f84b19e129f5c4a569686a42792
---
The architecture docs under `architecture/` reference pre-decomposition file paths, make stale claims about MCP behavior and entity capabilities, and mix runbook/CI procedure detail with system architecture. After the decomposition batch (article, drug, disease, trial, variant, markdown, CLI), every file-path reference in the architecture corpus is stale while concept-level descriptions remained accurate. The `make test-contracts` assertion against these docs is also failing because the docs no longer match reality.

Imported from March ticket 213. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/213-refresh-architecture-docs-for-post-decomposition-state
