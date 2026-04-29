# MCP Surface

BioMCP exposes the same biomedical command surface through stdio MCP and
Streamable HTTP. These canaries keep the transport entrypoints, probe routes,
and remote tool execution honest without re-encoding the whole MCP test suite.

## Stdio Entry Points Stay Guided

`mcp` and `serve` are both documented stdio entrypoints. The user-visible
contract here is that one remains the canonical stdio command and the other
stays the Claude Desktop-friendly alias.

```bash
mcp_help="$(../../tools/biomcp-ci mcp --help)"
echo "$mcp_help" | mustmatch like "Run MCP server over stdio"
echo "$mcp_help" | mustmatch like "Usage: biomcp mcp"
serve_help="$(../../tools/biomcp-ci serve --help)"
echo "$serve_help" | mustmatch like 'Alias for `mcp`'
echo "$serve_help" | mustmatch like "Usage: biomcp serve"
```

## Manual Stdio Startup Points Operators to HTTP

When an operator launches a stdio entrypoint without an MCP client, BioMCP
should fail closed but still explain the recovery path. Both spellings should
print the same stderr guidance and keep stdout free for MCP protocol traffic.

```bash
for cmd in mcp serve; do
  stdout_file="$(mktemp)"
  stderr_file="$(mktemp)"
  set +e
  ../../target/release/biomcp "$cmd" </dev/null >"$stdout_file" 2>"$stderr_file"
  status=$?
  set -e
  test "$status" -ne 0
  test ! -s "$stdout_file"
  stderr="$(cat "$stderr_file")"
  echo "$stderr" | mustmatch like "expects an MCP client on stdin"
  echo "$stderr" | mustmatch like "biomcp serve-http"
  echo "$stderr" | mustmatch not like "connection closed: initialized request"
done
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

## Read-Only Boundaries and Charted Calls Stay Visible

The transport should still reject CLI-only filesystem commands while returning
ordinary study text plus inline SVG for chart-safe read-only calls.

```bash
port=39089
../../tools/biomcp-ci serve-http --host 127.0.0.1 --port "$port" >/tmp/biomcp-mcp-boundary.log 2>&1 &
pid=$!; trap 'kill "$pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 40); do curl -fsS "http://127.0.0.1:$port/health" >/dev/null && break; sleep 0.25; done
out="$(uv run --no-sync python3 - "$port" <<'PY'
import asyncio
import sys
from datetime import timedelta
from mcp import ClientSession, types
from mcp.client.streamable_http import streamable_http_client

def mime_type(content):
    return getattr(content, "mimeType", getattr(content, "mime_type", None))

async def main(port: str) -> None:
    async with streamable_http_client(f"http://127.0.0.1:{port}/mcp", terminate_on_close=False) as (read_stream, write_stream, _):
        async with ClientSession(read_stream, write_stream, read_timeout_seconds=timedelta(seconds=30)) as session:
            await session.initialize()
            reject = await session.call_tool("biomcp", arguments={"command": "biomcp cache path"})
            chart = await session.call_tool("biomcp", arguments={"command": "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar"})
            print(next(c.text for c in reject.content if isinstance(c, types.TextContent)))
            print(next(c.text.splitlines()[0] for c in chart.content if isinstance(c, types.TextContent)))
            print(f"IMAGE: {next(mime_type(c) for c in chart.content if isinstance(c, types.ImageContent))}")

asyncio.run(main(sys.argv[1]))
PY
)"
echo "$out" | mustmatch like "CLI-only over MCP"
echo "$out" | mustmatch like "workstation-local filesystem paths"
echo "$out" | mustmatch like "# Study Mutation Frequency: TP53 (msk_impact_2017)"
echo "$out" | mustmatch like "IMAGE: image/svg+xml"
```
