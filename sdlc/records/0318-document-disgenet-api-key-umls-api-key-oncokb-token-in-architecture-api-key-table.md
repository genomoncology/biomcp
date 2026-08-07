---
base: 911463d9b5240f1f7cf1a9cea19f6888225cc1b5
head: 9d3b8a2bc7a28085e1b5537685c927d366ded04f
---
`architecture/technical/overview.md` lines 206-211 list five API keys (`NCBI_API_KEY`, `S2_API_KEY`, `OPENFDA_API_KEY`, `NCI_API_KEY`, `ALPHAGENOME_API_KEY`) but the codebase also reads `DISGENET_API_KEY`, `UMLS_API_KEY`, and `ONCOKB_TOKEN` — they are unset by `tools/biomcp-ci` which proves they are real env vars BioMCP consumes but they are undocumented in the operator-facing key table. Operators provisioning keys for a fresh install will miss these.

Imported from March ticket 318. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/318-document-disgenet-api-key-umls-api-key-oncokb-token-in-architecture-api-key-table
