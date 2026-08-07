---
base: 7578f0e972ff74400f7fcdaceb5faf1a3a63a561
head: 5646fa70e46af83e9f6747cf14b7c031340ac61b
---
`make spec` flaked once in `spec/surface/mcp.md` at "Remote Workflow Calls Keep BioMCP Text" because `biomcp serve-http --port 39088` could not bind — the fixed port was already in use, and the curl probe then retried against no server (issue 420). A rerun passed once the port was free. Fixed ports flake on shared machines; the spec must allocate its port instead.

Imported from March ticket 471. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/471-replace-fixed-mcp-spec-port-with-dynamic-reserved-port
