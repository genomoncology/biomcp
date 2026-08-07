---
flow: build
priority: 5
---
# Document who-ivd sync in MCP read-only contract

`src/mcp/shell.rs` rejects `who-ivd sync` (and the contract test passes), but `docs/reference/mcp-server.md` and `spec/15-mcp-runtime.md` still enumerate an older blocked set that omits the WHO IVD sync command. MCP consumers reading the durable contract don't know WHO IVD sync is blocked until they try it.

Completed under March on 2026-04-22, as March ticket 273. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/273-document-who-ivd-sync-in-mcp-read-only-contract
