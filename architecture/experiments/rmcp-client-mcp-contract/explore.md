# Explore — rmcp Rust client for BioMCP MCP contract

## Spike Question

Can BioMCP drop the Python `mcp` test-client dependency by verifying the same MCP contract from Rust using an rmcp client?

## Prior Art Summary

The current pytest contract uses Python `mcp` only as a black-box client. `tests/conftest.py` spawns `biomcp serve`, opens a Python `ClientSession`, and exposes the initialized session to `tests/test_mcp_contract.py`. `tests/test_mcp_http_transport.py` does the same for `serve-http` by starting the release binary, waiting for `/health`, then connecting to `/mcp` with the Python streamable HTTP client.

The Rust server already owns the protocol surface in `src/mcp/shell.rs`: one `BioMcpServer` advertises tools/resources/instructions, serves the `biomcp` tool, lists/reads resources, and is exposed over both stdio and streamable HTTP. That makes a black-box rmcp client test the closest replacement for the Python client.

rmcp source review found the needed client methods and transports: `TokioChildProcess`, `StreamableHttpClientTransport::from_uri`, and peer methods for `list_tools`, `call_tool`, `list_resources`, and `read_resource`. The worktree's `Cargo.toml` says `rmcp = "1.1.1"`, but Cargo resolves that caret requirement to `rmcp 1.7.0` in `Cargo.lock`; the effective server crate is already 1.7.0.

## Approaches Tried

### 1. Renamed rmcp 1.7 dev-dependency

I first tried a separate renamed dev-dependency:

```toml
rmcp-client = { package = "rmcp", version = "1.7.0", default-features = false, features = ["client", "transport-child-process", "transport-streamable-http-client-reqwest"] }
```

Cargo rejected this because the package was already a direct dependency under the name `rmcp`: `depends on crate rmcp v1.7.0 multiple times with different names`.

Result: not viable.

### 2. Same rmcp package with dev-only client features

I added a normal dev-dependency on the same `rmcp` package and feature set:

```toml
[dev-dependencies]
rmcp = { version = "1.1.1", features = ["client", "transport-child-process", "transport-streamable-http-client-reqwest"] }
```

Then I added `tests/rmcp_client_contract.rs` with two small tests:

- `rmcp_child_process_client_verifies_stdio_core_contract`
  - spawns `CARGO_BIN_EXE_biomcp serve` through `TokioChildProcess`
  - verifies initialize capabilities/instructions, `list_tools`, `call_tool("biomcp", {"command":"biomcp version"})`, `list_resources`, and `read_resource("biomcp://help")`
- `rmcp_streamable_http_client_verifies_core_contract`
  - starts `serve-http`, waits for `/health`, connects `StreamableHttpClientTransport::from_uri(".../mcp")`
  - verifies initialize/list_tools/call_tool/list_resources

Both child processes set `RUST_MIN_STACK=8388608`. Without that, the first debug prototype saw stack overflows in Tokio worker threads. With it, the focused nextest run is stable.

Measurements recorded in `architecture/experiments/rmcp-client-mcp-contract/results/rmcp-client-contract.json`:

```text
cargo test --no-run --test rmcp_client_contract: pass
cargo nextest run --test rmcp_client_contract: pass
2 tests run: 2 passed
stdio test runtime: 0.023s
HTTP test runtime: 0.276s
```

Dependency/feature findings:

- Effective rmcp version: `1.7.0` in `Cargo.lock`.
- Dev features added: `client`, `transport-child-process`, `transport-streamable-http-client-reqwest`.
- New lock packages observed from the client transport path: `process-wrap 9.1.0`, `reqwest 0.13.4`, `wasm-streams 0.5.0`.
- Existing production `reqwest 0.12.x` remains because BioMCP already uses it. rmcp's HTTP client currently pulls `reqwest 0.13.4`, so the test build carries two reqwest major/minor lines.
- Because the features are added under `[dev-dependencies]`, release builds should not need the client transports. Follow-up should verify with a clean release build before deleting Python deps.

### 3. Coverage inventory

| Existing assertion area | Portable to rmcp client? | Notes |
|---|---:|---|
| `initialize` has tools/resources capabilities | Yes | Covered in prototype for stdio and HTTP. |
| `instructions` contains public-source/suggest/skill text and omits old phrases | Yes | Covered in prototype core helper. |
| `list_tools` contains `biomcp`, omits `shell` | Yes | Covered in prototype. |
| tool annotations title `BioMCP`, `readOnlyHint` true | Yes | Covered in prototype. |
| tool description matches command-reference contract | Yes | Direct `tool.description` string checks in Rust. Not fully ported in prototype. |
| `call_tool` returns text chunks and no images for a text-only command | Yes | Covered in prototype with `biomcp version`. |
| `call_tool` read-only allowed commands (`skill list`, `discover`) | Yes | Same `CallToolRequestParams` and `RawContent` checks. |
| blocked mutating commands return `is_error` and text message | Yes | Same `CallToolResult.is_error` and text checks. |
| charted study call returns text then SVG image | Yes | rmcp exposes `RawContent::Image` with MIME/data. Needs fixture env port. |
| charted output-file rejection | Yes | Same call/result checks. |
| `list_resources` exact `biomcp://help` + skill inventory | Yes | Prototype proves list works; exact inventory is mechanical. |
| `read_resource` markdown contents and MIME | Yes | Prototype proves help read works; loop over all resources is mechanical. |
| invalid resource URI raises MCP error code/message | Yes | Peer call returns `ServiceError`; follow-up should assert error data shape. |
| HTTP initialize/list/call/resources over `/mcp` | Yes | Prototype passes. |
| markdown spec inline Python remote workflow call | Yes | Replace inline Python block with a Rust helper binary/test script or a small committed Rust example invoked by mustmatch. |
| markdown spec inline Python read-only/chart visibility | Yes | Same helper can print the exact text/image markers that mustmatch expects. |
| HTTP probe routes, host-header checks, make-target checks | Not about Python MCP | Already shell/curl/mustmatch; no rmcp migration needed. |

## Decision

Winner: migrate the MCP client contract to rmcp Rust tests/spec helpers.

Why:

- Stdio proof-of-life works with a real Cargo integration test against `biomcp serve`.
- Streamable HTTP also works against `serve-http` at `/mcp`.
- Resource listing and reading work through rmcp peer methods.
- Every Python SDK assertion maps to rmcp model types or to existing shell/curl checks. I did not find a contract area that requires Python `mcp` specifically.

Do not delete Python `mcp` after only the pytest port. The dependency can leave `[project.optional-dependencies].dev` only after both pytest imports and the inline Python blocks in `spec/surface/mcp.md` are gone.

## Outcome

promote

## Risks for Exploit

- The prototype needed `RUST_MIN_STACK=8388608` for spawned debug BioMCP processes. The build ticket should keep that in the test helper or find/fix the underlying stack pressure.
- rmcp streamable HTTP client pulls `reqwest 0.13.4` while BioMCP already uses `reqwest 0.12.x`; that is a dev/test cost, but follow-up should confirm release builds do not include the extra client stack.
- Markdown specs need a clean replacement shape. Best follow-up shape is a small Rust test/helper that prints stable mustmatch text for the two inline Python blocks, then remove `prepare_mcp_markdown_deps` from `scripts/run-specs.sh`.
- The full port still needs exact assertion parity for descriptions, chart fixtures, invalid-resource error shape, and all blocked-command messages.

## Follow-up Build Ticket Shape

- Add/keep rmcp client dev features in `Cargo.toml`.
- Expand `tests/rmcp_client_contract.rs` until it covers all assertions currently in `tests/test_mcp_contract.py` and `tests/test_mcp_http_transport.py`.
- Replace inline Python in `spec/surface/mcp.md` with a committed Rust helper/script that uses rmcp client and prints the same mustmatch markers.
- Remove all `from mcp` imports from tests/spec support.
- Remove `mcp>=1.0.0` from `pyproject.toml` dev extras and refresh `uv.lock`.
- Remove bounded MCP markdown Python setup (`prepare_mcp_markdown_deps`) from `scripts/run-specs.sh` once no spec imports Python `mcp`.
- Run `make lint`, `make test`, and `make spec`.
