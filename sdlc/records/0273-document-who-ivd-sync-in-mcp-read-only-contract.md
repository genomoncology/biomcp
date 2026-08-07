---
base: 1025e1017fb1265d5d5c6dfa96a7cc4c0c0e7830
head: e05b81701317c6a7f4a4627ba48197f9743335a2
---
`src/mcp/shell.rs` rejects `who-ivd sync` (and the contract test passes), but `docs/reference/mcp-server.md` and `spec/15-mcp-runtime.md` still enumerate an older blocked set that omits the WHO IVD sync command. MCP consumers reading the durable contract don't know WHO IVD sync is blocked until they try it.

Imported from March ticket 273. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/273-document-who-ivd-sync-in-mcp-read-only-contract
