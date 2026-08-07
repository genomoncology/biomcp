---
flow: build
priority: 6
---
# Return chart SVG inline via MCP as base64 image

Agents using BioMCP through MCP (Claude Desktop, etc.) cannot currently see charts — Kuva renders to terminal or writes SVG files, neither of which is visible through MCP's read-only tool interface. MCP's tool response format supports `image` content type with base64-encoded data. Since Kuva already produces SVG strings in memory before writing to disk, returning the SVG inline as a base64 image in the MCP response would unlock charting for all agent workflows without requiring file system access.

Completed under March on 2026-03-18, as March ticket 011. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/011-mcp-inline-chart-svg

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
