---
base: ce75d78960d744ed0d6a6a6f493adf41922aa7ce
head: 99643e3d0bea348235630e775a486c7dcaa36fd8
---
MCP admits article full-text retrieval and can return `full_text_path` / `Saved to: <absolute path>`, disclosing server cache/workstation layout to remote clients. Cache commands are already blocked because local paths are sensitive.

Imported from March ticket 556. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/556-redact-workstation-local-full-text-paths-from-mcp-responses
