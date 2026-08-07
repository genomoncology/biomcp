---
base: 73ea6d79a759c1b8e88252502788e55dff876684
head: 2f2bdc562336ab28d981f770a50743abe02c8724
---
Hand-writing MCP client config (JSON blocks, command lines) is a common adoption failure point. A `biomcp mcp-config --client <name>` command that prints the exact, correct config block for a given client removes that friction and keeps the snippets always-correct against the installed binary.

Imported from March ticket 477. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/477-add-biomcp-mcp-config-client-config-generator-command
