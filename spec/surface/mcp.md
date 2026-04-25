# MCP Surface

BioMCP exposes the same biomedical command surface through stdio MCP and
Streamable HTTP. These canaries keep the transport entrypoints, probe routes,
and remote tool execution honest without re-encoding the whole MCP test suite.

## Stdio Entry Points Stay Guided

`mcp` and `serve` are both documented stdio entrypoints. The user-visible
contract here is that one remains the canonical stdio command and the other
stays the Claude Desktop-friendly alias.

```bash
mcp_help="$("$BIOMCP_BIN" mcp --help)"
echo "$mcp_help" | mustmatch like "Run MCP server over stdio"
echo "$mcp_help" | mustmatch like "Usage: biomcp mcp"
serve_help="$("$BIOMCP_BIN" serve --help)"
echo "$serve_help" | mustmatch like 'Alias for `mcp`'
echo "$serve_help" | mustmatch like "Usage: biomcp serve"
```

## Streamable HTTP Help Names the Canonical Route

The remote/server deployment mode should keep pointing operators at `/mcp` and
the lightweight probe routes rather than drifting back toward legacy SSE copy.

```bash
out="$(../../tools/biomcp-ci serve-http --help)"
echo "$out" | mustmatch like "Streamable HTTP server at /mcp"
echo "$out" | mustmatch like "GET /health, GET /readyz, GET /."
echo "$out" | mustmatch like "--host <HOST>"
```

## Probe Routes Stay Lightweight

The HTTP surface is intentionally tiny: two readiness probes and one root
descriptor that advertises the streamable transport and canonical MCP path.

```bash
port=39087
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-routes.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
sleep 2
curl -fsS "http://127.0.0.1:$port/health" | mustmatch like '"status":"ok"'
curl -fsS "http://127.0.0.1:$port/readyz" | mustmatch like '"status":"ok"'
root="$(curl -fsS "http://127.0.0.1:$port/")"
echo "$root" | mustmatch like '"transport":"streamable-http"'
echo "$root" | mustmatch like '"mcp":"/mcp"'
```

## Remote Workflow Calls Keep BioMCP Text

The remote tool should execute normal BioMCP workflows, not collapse them into
an MCP-specific summary. The streamable-HTTP demo is a compact proof that the
server still returns ordinary BioMCP text over the transport.

```bash
port=39088
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-demo.log 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true' EXIT
sleep 2
out="$(uv run --quiet --script ../../examples/streamable-http/streamable_http_client.py "http://127.0.0.1:$port")"
echo "$out" | mustmatch like "Connecting to http://127.0.0.1:$port/mcp"
echo "$out" | mustmatch like 'Command: biomcp search all --gene BRAF --disease melanoma --counts-only'
echo "$out" | mustmatch like "Query: condition=melanoma, mutation=BRAF V600E"
```
