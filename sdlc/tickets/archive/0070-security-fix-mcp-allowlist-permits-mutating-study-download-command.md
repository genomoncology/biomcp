---
flow: build
priority: 9
---
# Security fix — MCP allowlist permits mutating study download command

The MCP server is documented and annotated as read-only (`read_only_hint = true`), but the command allowlist in `src/mcp/shell.rs` permits the entire `study` command family — including `study download`, which performs network I/O, creates directories, and extracts archives to the local filesystem. Any MCP client connecting to the server can invoke a write operation through what is advertised as a read-only interface.

Completed under March on 2026-03-27, as March ticket 070. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/070-security-fix-mcp-allowlist-permits-mutating-study-download-command
