---
flow: quickfix
priority: 5
---
# Replace fixed MCP spec port with dynamic reserved port

`make spec` flaked once in `spec/surface/mcp.md` at "Remote Workflow Calls Keep BioMCP Text" because `biomcp serve-http --port 39088` could not bind — the fixed port was already in use, and the curl probe then retried against no server (issue 420). A rerun passed once the port was free. Fixed ports flake on shared machines; the spec must allocate its port instead.

Completed under March on 2026-07-01, as March ticket 471. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/471-replace-fixed-mcp-spec-port-with-dynamic-reserved-port
