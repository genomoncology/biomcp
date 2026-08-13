---
flow: build
priority: 10
---
# Block server-local input from the raw MCP command tool

The raw `biomcp` MCP tool rejects `variant articles --input`, but its broad
variant allowlist still accepts `variant normalize car --input` and `variant
erepo --input`. Those commands open a path on the MCP server, and `--input -`
reads the server process's stdin. A remote MCP caller must not be able to make
the server read local files or consume the MCP protocol stream.

Keep the ordinary CLI file-input behavior. MCP callers that need CAR or ERepo
batch work must use the existing typed tools and provide bounded values in the
request itself. The raw-command policy must be exhaustive: adding another
file- or stdin-reading command later must fail a structural test until its MCP
behavior is decided explicitly.

## Done when

- Raw MCP execution rejects every CAR, ERepo, and variant-article `--input`
  spelling before command execution, including separate and `--input=...`
  forms and `-`.
- The rejection is a normal structured MCP tool error and does not open a
  path, read stdin, or disturb the protocol connection.
- Typed CAR and ERepo requests and ordinary terminal CLI file input continue
  to work.
- A structural test enumerates all shipped CLI arguments that can read a
  caller-selected file or stdin and proves the raw MCP allowlist owns each one.

## Authorized test changes

Design may restate the MCP allowlist assertions in `src/mcp/shell.rs`, the
typed MCP boundary assertions in `tests/test_mcp_tool_catalog.py` and
`tests/rmcp_client_contract.rs`, and the public MCP contract in
`spec/surface/mcp.md`.
