---
flow: quickfix
priority: 3
---
# Document DISGENET_API_KEY UMLS_API_KEY ONCOKB_TOKEN in architecture API key table

`architecture/technical/overview.md` lines 206-211 list five API keys (`NCBI_API_KEY`, `S2_API_KEY`, `OPENFDA_API_KEY`, `NCI_API_KEY`, `ALPHAGENOME_API_KEY`) but the codebase also reads `DISGENET_API_KEY`, `UMLS_API_KEY`, and `ONCOKB_TOKEN` — they are unset by `tools/biomcp-ci` which proves they are real env vars BioMCP consumes but they are undocumented in the operator-facing key table. Operators provisioning keys for a fresh install will miss these.

Completed under March on 2026-04-27, as March ticket 318. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/318-document-disgenet-api-key-umls-api-key-oncokb-token-in-architecture-api-key-table
