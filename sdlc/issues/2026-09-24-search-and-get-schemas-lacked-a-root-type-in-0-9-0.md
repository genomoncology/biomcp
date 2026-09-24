# search and get schemas lacked a root type in 0.9.0

Filed 2026-09-24 from GitHub #284.

## Symptom

In 0.9.0, `tools/list` returns `search` and `get` input schemas with `oneOf` and no `"type": "object"`. Claude Code rejects the whole list with `tools.1.inputSchema.type: Invalid input`, so the server connects with zero tools.

## Cause

`v0.9.0:src/mcp/shell.rs:291` built the search schema as `{"oneOf": branches}`. The get schema had the same gap.

## Resolved

Fixed on main by e559cae2 (2026-09-22, "Declare object roots on the typed MCP tool schemas"), before the report arrived. Tests pin it: `src/mcp/shell.rs:1727` and `:1730`, `tests/test_mcp_2026_protocol.py`, and the release smoke's tools/list check (`tests/test_release_smoke_script.py:54-60`). It ships in 0.9.1. No ticket. Ticket 1240 reshapes these schemas and must keep the root `type`. The existing tests enforce that.

After 0.9.1 is on PyPI, reply on #284 with the release and close it. The reply is public and needs Ian's OK.
