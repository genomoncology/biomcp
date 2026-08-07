---
flow: build
priority: 7
---
# Align BIOMCP env configuration docs with runtime and add parity ratchet

The architecture review found configuration truth drift: `docs/reference/configuration.md` documents `BIOMCP_CACHE_MAX_AGE`, but the reviewed cache config code does not read that environment variable; meanwhile `BIOMCP_GENE_GET_STRATEGY`, `BIOMCP_GENE_OPTIONAL_TIMEOUT_MS`, `BIOMCP_GENE_TIMING_PATH`, and `BIOMCP_DISABLE_KEGG` are live knobs but are not clearly classified as operator-supported, internal/test-only, or retired. Config drift creates false operator contracts.

Completed under March on 2026-06-30, as March ticket 465. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/465-align-biomcp-env-configuration-docs-with-runtime-and-add-parity-ratchet
