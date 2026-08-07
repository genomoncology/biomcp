---
flow: build
priority: 6
---
# Refresh architecture docs for post-decomposition state

The architecture docs under `architecture/` reference pre-decomposition file paths, make stale claims about MCP behavior and entity capabilities, and mix runbook/CI procedure detail with system architecture. After the decomposition batch (article, drug, disease, trial, variant, markdown, CLI), every file-path reference in the architecture corpus is stale while concept-level descriptions remained accurate. The `make test-contracts` assertion against these docs is also failing because the docs no longer match reality.

Completed under March on 2026-04-15, as March ticket 213. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/213-refresh-architecture-docs-for-post-decomposition-state
