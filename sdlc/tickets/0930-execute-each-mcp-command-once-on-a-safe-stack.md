---
flow: build
priority: 10
---
# Execute each MCP command once on a safe stack

The CLI owns a larger-stack execution boundary, but MCP awaits the same large
command future directly on a Tokio worker. An ordinary debug build aborts on
raw `version` and typed gene search with a stack overflow. Human-form MCP calls
also rerun the command in JSON mode to derive metadata, which can double
provider traffic and combine two different snapshots.

## Execution contract

Create one shared CLI-backed MCP execution boundary used by raw and typed tools
over stdio and HTTP. It provides the deliberate stack size without relying on
`RUST_MIN_STACK`, dispatches the command exactly once, and returns one
structured outcome from which human text, JSON, metadata, provenance,
pagination, and footer are projected. A command failure becomes one normal MCP
tool error and never aborts the server.

Do not run an async command future on an arbitrary MCP runtime worker and do
not execute a second JSON command to recover metadata. Chart rendering may
produce more than one projection from the one data snapshot, but it must not
retrieve or compute the underlying study command twice.

## Done when

- Fresh default-stack debug, spec, and release builds complete raw `version`,
  typed search, typed get, a locally failing provider command, and a chart call
  over both MCP transports without aborting.
- A counting local provider observes exactly one dispatch/request for human and
  JSON modes, including failure.
- Body and metadata are proven to come from one deliberately changing fixture
  snapshot.
- Tests unset `RUST_MIN_STACK`; no test or production launcher needs a global
  stack-size environment workaround.
- Concurrent tool calls remain independent and server shutdown does not leak
  execution threads.

## Authorized test changes

Design commits may restate MCP execution and response tests in
`src/cli/outcome.rs`, `src/mcp/shell.rs`, `crates/biomcp-mcp-contract-client`,
MCP specs, and local transport fixtures. Existing allowlist, path-redaction,
JSON, protocol-framing, and concurrency behavior remain covered.

The src line ceiling may rise by at most 220 lines.
