---
base: 4aa45eeb6745d7edf772d330045104f9597a3bd3
head: 13b1928167408985ad5e8092c3afabdf2c4ad37f
---

# Serve the stateless MCP 2026-07-28 protocol

BioMCP now serves the 2026-07-28 MCP revision over stdio and HTTP without a
handshake while preserving both legacy session revisions. Modern requests and
results enforce the revision's metadata, errors, identity, result type, cache,
and subscription contracts so discovery can advertise the revision truthfully.
