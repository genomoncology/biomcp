---
base: 74c5cb4a2bc5cc37b8fbbd2bad03557984b2c4f4
head: 80b9172c0b0fd9b1b0e6bcee52cbc33c8e75520c
---
Flow upstream source provenance (and next_commands/ladder) through the MCP boundary so every response carries per-section source attribution; reuses the existing _meta builders. High value, on the optimizer critical path.

Imported from March ticket 434. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/434-mcp-source-attribution-surface-upstream-provenance-through-the-mcp-boundary
