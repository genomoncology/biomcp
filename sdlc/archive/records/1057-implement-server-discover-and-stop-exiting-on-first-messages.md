---
base: 4897eca6d4a969b9443e2d571784467b065ff884
head: 586b709d441e400277126344463a215beb78d662
---

# Add pre-session MCP discovery

BioMCP stdio now answers `server/discover` with truthful legacy revisions,
capabilities, identity, and cache hints. Early requests return JSON-RPC errors
without ending the stream, preserving both legacy handshake paths.
