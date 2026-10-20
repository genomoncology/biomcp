---
base: fb6036969621f2c7e94f9e95b3d328adbf72a122
head: 81e506bb70c701cb695634b11002fb8c64789b87
---

# Authorize raw MCP commands from the parsed command

Raw MCP authorization and execution now share one parsed CLI command. This
prevents global flags from bypassing local-input, binary-download, or
redaction safeguards.

The explicit allowlist denies new command variants until they are reviewed,
while preserving safe read-only routes over stdio and HTTP.
