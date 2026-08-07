---
base: 16dd133ccc1ac97b4a14cddb433708142f0cf854
head: fe37826b512a481c980c8757d64251217938ab55
---
The MCP server is documented and annotated as read-only (`read_only_hint = true`), but the command allowlist in `src/mcp/shell.rs` permits the entire `study` command family — including `study download`, which performs network I/O, creates directories, and extracts archives to the local filesystem. Any MCP client connecting to the server can invoke a write operation through what is advertised as a read-only interface.

Imported from March ticket 070. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/070-security-fix-mcp-allowlist-permits-mutating-study-download-command
