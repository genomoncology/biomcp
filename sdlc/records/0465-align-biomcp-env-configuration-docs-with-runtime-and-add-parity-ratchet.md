---
base: 6e4423a003f1e1e63423ea7a8ae07386efd4456f
head: d87b910cdd6219ec0823a80c6dd2422f941faa0f
---
The architecture review found configuration truth drift: `docs/reference/configuration.md` documents `BIOMCP_CACHE_MAX_AGE`, but the reviewed cache config code does not read that environment variable; meanwhile `BIOMCP_GENE_GET_STRATEGY`, `BIOMCP_GENE_OPTIONAL_TIMEOUT_MS`, `BIOMCP_GENE_TIMING_PATH`, and `BIOMCP_DISABLE_KEGG` are live knobs but are not clearly classified as operator-supported, internal/test-only, or retired. Config drift creates false operator contracts.

Imported from March ticket 465. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/465-align-biomcp-env-configuration-docs-with-runtime-and-add-parity-ratchet
