---
flow: build
priority: 9
deps: ["0930"]
---
# Reject binary downloads over MCP

Trial documents and article assets are byte streams. The raw MCP escape hatch
currently admits their `get` forms and converts the bytes with lossy UTF-8,
which corrupts PDFs and other non-text files and can place tens of megabytes in
an agent context.

## Boundary contract

MCP rejects `get trial <id> document <filename>` and
`get article <id> asset <asset-key>` before command dispatch, for both the raw
tool and any typed route. The error says that binary downloads are CLI-only and
shows the corresponding redirection command. Manifests such as trial
`documents` and article `assets` remain available because they are bounded
structured text.

No MCP path may convert `CommandOutcome::Binary` with lossy UTF-8. Reaching a
binary outcome after allowlist validation is a safe internal tool error, not a
text response. A future MCP Resource or ResourceLink design requires its own
ticket with byte, media-type, lifetime, and client-capability bounds.

## Done when

- Raw and typed document/asset download attempts fail before a counting local
  server observes a request.
- Non-UTF-8 fixture bytes are never present as replacement-character text in an
  MCP response.
- Manifest calls still work and remain within their existing response budgets.
- Error text contains no local path and gives one valid CLI command.
- One MCP call can perform at most one download even if an allowlist regression
  reaches the execution boundary.

## Authorized test changes

Design and code commits may add MCP allowlist, outcome, and process tests in
`src/mcp/shell.rs`, `src/cli/outcome.rs`, trial-document and article-asset local
fixtures, and `spec/surface/mcp.md`. Existing CLI byte-for-byte downloads and
MCP text/JSON outcomes remain covered.

The src line ceiling may rise by at most 100 lines.
