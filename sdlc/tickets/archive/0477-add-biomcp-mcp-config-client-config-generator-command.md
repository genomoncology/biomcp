---
flow: build
priority: 5
---
# Add biomcp mcp-config client config generator command

Hand-writing MCP client config (JSON blocks, command lines) is a common adoption failure point. A `biomcp mcp-config --client <name>` command that prints the exact, correct config block for a given client removes that friction and keeps the snippets always-correct against the installed binary.

Completed under March on 2026-07-01, as March ticket 477. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/477-add-biomcp-mcp-config-client-config-generator-command
