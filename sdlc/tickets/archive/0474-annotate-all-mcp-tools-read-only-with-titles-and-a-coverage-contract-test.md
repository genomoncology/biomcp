---
flow: build
priority: 5
---
# Annotate all MCP tools read-only with titles and a coverage contract test

BioMCP is entirely read-only federation, but its MCP tools are not annotated as such. MCP supports tool annotations (`readOnlyHint`, `title`, plus a clear `description`) that clients surface in their tool pickers and that directories display — and that help the model pick the right tool. Marking every tool read-only is both a trust signal (it matches BioMCP's core "no writes to external systems" principle) and a discovery/usability win.

Completed under March on 2026-07-01, as March ticket 474. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/474-annotate-all-mcp-tools-read-only-with-titles-and-a-coverage-contract-test
