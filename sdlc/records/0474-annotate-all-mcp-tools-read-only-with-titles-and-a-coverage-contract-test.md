---
base: 7ae7856c986c91385fe82559f8655eea05c58059
head: 97c72d801f4550fd8f24d7ee58b36456257d3eb6
---
BioMCP is entirely read-only federation, but its MCP tools are not annotated as such. MCP supports tool annotations (`readOnlyHint`, `title`, plus a clear `description`) that clients surface in their tool pickers and that directories display — and that help the model pick the right tool. Marking every tool read-only is both a trust signal (it matches BioMCP's core "no writes to external systems" principle) and a discovery/usability win.

Imported from March ticket 474. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/474-annotate-all-mcp-tools-read-only-with-titles-and-a-coverage-contract-test
