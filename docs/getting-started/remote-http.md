# Remote Streamable HTTP Server

Use `biomcp serve-http` when you need one shared MCP server for remote clients,
multiple agent workers, or a container/network deployment.

Use `biomcp serve` when a single local client such as Claude Desktop or Cursor
launches BioMCP over stdio.

## Start the server

```bash
biomcp serve-http --host 127.0.0.1 --port 8080
```

Use `--host 0.0.0.0` only when the server must accept connections from other
machines or containers. A non-loopback bind must name the Host values BioMCP
will receive:

```bash
biomcp serve-http --host 0.0.0.0 --port 8080 \
  --allowed-hosts biomcp.example.org,localhost:8080
```

Use TLS and authentication at a trusted reverse proxy, gateway, or private
network boundary. The Host allowlist prevents unexpected Host headers; it is
not authentication. If a proxy rewrites `Host`, allow the value BioMCP actually
receives. BioMCP does not trust forwarding headers to infer it.

## MCP endpoint and probes

The canonical MCP endpoint is `/mcp`. Probe routes are `/health`, `/readyz`,
and `/`.

| Route | Purpose |
|-------|---------|
| `POST /mcp` | Streamable HTTP MCP requests |
| `GET /mcp` | Streamable HTTP session stream |
| `GET /health` | Liveness check returning `{"status":"ok"}` |
| `GET /readyz` | Readiness check returning `{"status":"ok"}` |
| `GET /` | BioMCP identity document with name, version, transport, and MCP path |

## Minimal Python client

```python
import asyncio
from datetime import timedelta

from mcp import ClientSession
from mcp.client.streamable_http import streamable_http_client


async def main() -> None:
    async with streamable_http_client(
        "http://127.0.0.1:8080/mcp",
        terminate_on_close=False,
    ) as (r, w, _):
        async with ClientSession(
            r,
            w,
            read_timeout_seconds=timedelta(seconds=30),
        ) as session:
            result = await session.initialize()
            print(result.serverInfo)


asyncio.run(main())
```

## Runnable demo

The repo includes a standalone demo you can run directly. It keeps the
Streamable HTTP connectivity proof, then runs a three-step BRAF V600E melanoma
workflow over the remote MCP `biomcp` tool:

- `biomcp search all --gene BRAF --disease melanoma --counts-only`
- `biomcp get variant "BRAF V600E" clinvar`
- `biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5`

```bash
biomcp serve-http --host 127.0.0.1 --port 8080
uv run --script examples/streamable-http/streamable_http_client.py
uv run --script examples/streamable-http/streamable_http_client.py http://127.0.0.1:8080
```

The demo connects to `/mcp`, prints `Command: ...` before each BioMCP step,
and leaves the real markdown output untouched so a screenshot or recording
still makes sense without extra narration.

See `examples/streamable-http/README.md` for the short newcomer walkthrough, expected output
markers, `uv run --quiet` guidance for first-run dependency noise, and the
release-binary note for repo verification.

The examples above disable explicit session termination because the current
Python MCP client logs a warning when the server acknowledges teardown with
HTTP `202 Accepted` (`terminate_on_close=False` in the demo client).

## Related docs

- [Claude Desktop (stdio setup)](claude-desktop.md)
- [MCP Server Reference](../reference/mcp-server.md)
- [RUN.md](https://github.com/genomoncology/biomcp/blob/main/RUN.md)
