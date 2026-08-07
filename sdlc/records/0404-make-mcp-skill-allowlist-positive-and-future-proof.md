---
base: b7761444d07d078d79ecc33ab2d4e38b66b08abd
head: fe022797b37be8d461574f350708ac4bbeeef103
---
The MCP guard is broadly positive-allowlist based, but `skill` currently allows every subcommand except `install`. That is safe only while future skill subcommands remain read-only by accident. MCP is a read-only surface and should fail closed when the CLI family grows.

Imported from March ticket 404. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/404-make-mcp-skill-allowlist-positive-and-future-proof
