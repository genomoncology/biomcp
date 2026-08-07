---
flow: quickfix
priority: 10
---
# Redact workstation-local full-text paths from MCP responses

MCP admits article full-text retrieval and can return `full_text_path` / `Saved to: <absolute path>`, disclosing server cache/workstation layout to remote clients. Cache commands are already blocked because local paths are sensitive.

Completed under March on 2026-07-15, as March ticket 556. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/556-redact-workstation-local-full-text-paths-from-mcp-responses
