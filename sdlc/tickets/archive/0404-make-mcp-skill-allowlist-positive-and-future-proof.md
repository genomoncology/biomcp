---
flow: quickfix
priority: 9
---
# Make MCP skill allowlist positive and future-proof

The MCP guard is broadly positive-allowlist based, but `skill` currently allows every subcommand except `install`. That is safe only while future skill subcommands remain read-only by accident. MCP is a read-only surface and should fail closed when the CLI family grows.

Completed under March on 2026-06-08, as March ticket 404. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/404-make-mcp-skill-allowlist-positive-and-future-proof
