---
flow: quickfix
priority: 8
---

# Make MCP footprint claims truthful

Public pages must not present the historical 6,707-byte and 1,628-token catalog as current. Label that pair as the 0932 historical snapshot, and describe the current catalog through the canonical measurement command and enforced ceilings of 16,000 bytes and 4,000 tokens. Do not raise those ceilings or hand-copy another mutable current measurement.

Restatements are authorized in `docs/blog/we-deleted-35-tools.md`, `docs/getting-started/claude-desktop.md`, `docs/reference/mcp-server.md`, `tests/test_mcp_tool_catalog.py`, and `tests/test_docs_changelog_refresh.py`.
