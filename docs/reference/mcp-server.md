# MCP Server Reference

BioMCP can run as a local stdio MCP server or as a remote Streamable HTTP MCP server. Both transports expose the same read-only biomedical tools and resources; choose the transport based on where the MCP client runs.

## Official MCP Registry

BioMCP's official MCP Registry name is `io.github.genomoncology/biomcp`. The
repo-root `server.json` is the metadata source that points registry clients at
the `biomcp-cli` PyPI package and the existing `biomcp serve` stdio command.

Release operators publish or refresh the registry entry with the official
publisher CLI:

```bash
mcp-publisher init
mcp-publisher login github
mcp-publisher publish
```

Use GitHub namespace authentication for `io.github.genomoncology/biomcp`; it
proves ownership through the `genomoncology/biomcp` repository. Each release
must keep `server.json` on the release version before publishing. The release
workflow stamps `server.json` from the tag alongside the other package manifests.

## Which server mode should I use?

| Use case | Command | Transport | Notes |
|---|---|---|---|
| Local desktop client, such as Claude Desktop | `biomcp serve` | stdio | The MCP client starts BioMCP and talks over stdin/stdout. `biomcp mcp` is the same legacy alias. |
| Remote or containerized deployment | `biomcp serve-http --host <host> --port <port>` | Streamable HTTP | Exposes MCP at `/mcp` and probe routes at `/health`, `/readyz`, and `/`. |

Manual stdio runs require an MCP client to send the initialize handshake on stdin. If `biomcp serve` or `biomcp mcp` is launched with stdin closed, the command exits non-zero and prints recovery guidance that points operators to `biomcp serve-http` for manual testing.

## Local stdio server

Use stdio when BioMCP runs on the same machine as the MCP client:

```bash
biomcp serve
```

A desktop MCP configuration usually points directly at the installed `biomcp` binary and passes `serve` as the argument. The server writes MCP protocol messages to stdout, so do not wrap this command in scripts that print banners or other text to stdout.

## Remote HTTP server

Use Streamable HTTP when the MCP client reaches BioMCP over a network, through a container port, or behind a proxy:

```bash
biomcp serve-http --host 0.0.0.0 --port 8000
```

Routes:

- `/mcp` — MCP Streamable HTTP endpoint.
- `/health` — lightweight health probe.
- `/readyz` — readiness probe.
- `/` — small index/help response for humans and load balancers.

Point HTTP-capable MCP clients at the full MCP URL, for example `https://biomcp.example.org/mcp` after your gateway terminates TLS.

## Host guard and proxies

As of BioMCP 0.8.24, the HTTP Host guard is opt-in. If you do not pass `--allowed-hosts`, BioMCP accepts any Host header. That default is intentional so the server works behind containers, reverse proxies, service meshes, and custom domains without extra configuration. This behavior corresponds to issue #240.

Use `--allowed-hosts` only when BioMCP itself should reject unexpected Host headers:

```bash
biomcp serve-http --host 0.0.0.0 --port 8000 \
  --allowed-hosts biomcp.example.org,localhost:8000
```

If a proxy rewrites Host headers, include the value BioMCP actually receives or leave the option unset and enforce host policy at the proxy.

## Authentication model

BioMCP's HTTP MCP transport is unauthenticated by design. It does not implement user login, bearer-token validation, OAuth, session cookies, or per-user authorization.

For remote deployment, put BioMCP behind infrastructure you control, such as:

- an API gateway,
- a reverse proxy with SSO or mTLS,
- a private network/VPN,
- a platform ingress that enforces authentication before forwarding to BioMCP.

Keep BioMCP bound to a private interface when possible, and expose only the authenticated gateway to users.

## Provider API keys

Provider keys for built-in tools are environment variables read by the BioMCP process. Configure them in the service manager, container environment, or desktop MCP configuration that launches BioMCP.

Common keys include `ONCOKB_TOKEN`, `ALPHAGENOME_API_KEY`, `NCI_API_KEY`, `NCBI_API_KEY`, `S2_API_KEY`, `OPENFDA_API_KEY`, and `UMLS_API_KEY`. See the [API Keys guide](../getting-started/api-keys.md) for the current list and source-specific behavior.

## MCP tools and resources

The MCP server advertises both tools and resources.

### Typed tools

Prefer the typed tools when possible:

- `search` for biomedical searches across supported entity types.
- `get` for record lookup and sectioned detail retrieval.

Their schemas enumerate valid entity names, valid get section tokens, and the bounded search `limit`.

### Raw command escape hatch

The raw `biomcp` tool remains available for read-only CLI commands outside the first typed slice. It is an escape hatch, not the preferred first call. It accepts read-only commands such as `suggest`, `discover`, `biomcp skill list`, `biomcp skill render`, embedded `biomcp skill <number-or-slug>` lookups, and the catalog-only `study download --list` form.

Mutating or workstation-local commands are blocked in MCP mode. Examples include `skill install`, local source sync commands, `update`, `uninstall`, and `study download <study_id>`. Cache-family commands such as `cache path`, `cache stats`, `cache clean`, and `cache clear` are also rejected because they reveal workstation-local paths and filesystem context.

### Resources

Current builds always publish the help resource and one markdown resource per embedded skill use-case.

| URI | Name | Notes |
|---|---|---|
| `biomcp://help` | BioMCP Overview | Same in-memory help text returned by `biomcp skill render`. |
| `biomcp://skill/<slug>` | Embedded worked example | Markdown resource for a built-in BioMCP skill use-case. |

Workflow ladders do not add MCP resources. MCP callers that execute BioMCP commands with `--json`, or pass tool input `json: true`, receive the same CLI JSON contract, so first-call responses can include `_meta.workflow` and `_meta.ladder[]` when a sidecar-backed ladder trigger matches. `_meta.next_commands` remains the dynamic one-hop follow-up list.

## Tool responses

By default, the `biomcp` tool keeps non-chart calls as readable text and appends compact provenance when CLI JSON exposes it:

- `## Sources` rolls up `_meta.section_sources` per section.
- `## Next commands` rolls up `_meta.next_commands` as copyable follow-up commands.

Agents that need the full structured contract can pass the tool input field `json: true`; BioMCP injects `--json` and returns CLI JSON text with metadata such as `_meta.section_sources`, `_meta.evidence_urls`, `_meta.next_commands`, and `_meta.ladder`.

In MCP mode, charted `study` commands return two success content blocks in order:

1. `text` with the normal markdown/table output.
2. `image` with `mimeType = "image/svg+xml"` and base64-encoded SVG data.

MCP chart calls do not write files. If the caller supplies `--output` or `-o`, the tool returns an error instructing the caller to consume the inline image instead.

## Operational checklist

- Use `biomcp serve` for local stdio clients.
- Use `biomcp serve-http` for remote Streamable HTTP clients and route them to `/mcp`.
- Probe `/health`, `/readyz`, or `/` from load balancers and deployment checks.
- Leave `--allowed-hosts` unset unless BioMCP itself should enforce Host headers.
- Put remote HTTP deployments behind your own authentication and TLS layer.
- Set provider keys as environment variables for the BioMCP process.
