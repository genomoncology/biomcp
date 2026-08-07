---
base: 68c3f8187053d5b06008e320354c3502c4d6fe9d
head: da56d0800204a7116cdd9ca796c9697b231b59c9
---
Ticket 110 was split at design-review because it bundled new CLI family implementation with multi-zone public contract cutover. This is child 110A: ship `biomcp cache path` as the canonical operator discovery surface for the managed HTTP cache path, with the MCP boundary and all affected docs/specs updated in the same slice.

Imported from March ticket 116. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/116-cache-path-cli-cutover
