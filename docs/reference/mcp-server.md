# MCP Server Reference

BioMCP can run as a local stdio MCP server or as a remote Streamable HTTP MCP server. Both transports expose the same read-only biomedical tools and resources; choose the transport based on where the MCP client runs.

## Official MCP Registry

BioMCP's official MCP Registry name is `io.github.genomoncology/biomcp`. The
committed `server.json` is the registry metadata source: it points
clients at the `biomcp-cli` PyPI package and the existing `biomcp serve` stdio
command. The workflow never stamps registry metadata from a tag.

The protected release workflow never stamps registry metadata from a tag and
does not submit BioMCP to the official MCP Registry. After a promoted release
passes its public-artifact checks, an operator must separately review the
committed metadata and submit it to the official registry. That manual action
must not be described as complete until the registry accepts it. The committed
metadata remains the truthful record for the already published v0.8.25 release.

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

### Docker stdio server

Use the GHCR image when the MCP client can launch Docker instead of a local binary:

```bash
docker run --rm -i ghcr.io/genomoncology/biomcp serve
```

Pass provider keys through the container environment, for example with `-e ONCOKB_TOKEN`, `-e NCBI_API_KEY`, or the equivalent environment setting in your MCP client. The `-e NAME` form forwards the value from your shell without writing the secret into the command.

## Remote HTTP server

Use Streamable HTTP when the MCP client reaches BioMCP over a network, through a container port, or behind a proxy:

```bash
biomcp serve-http --host 0.0.0.0 --port 8000 \
  --allowed-hosts biomcp.example.org
```

Routes:

- `/mcp` — MCP Streamable HTTP endpoint. Each `POST /mcp` encoded request body may contain at most 65,536 bytes; larger fixed-length and streamed requests receive HTTP 413.
- `/health` — lightweight health probe.
- `/readyz` — readiness probe.
- `/` — small index/help response for humans and load balancers.

Point HTTP-capable MCP clients at the full MCP URL, for example `https://biomcp.example.org/mcp` after your gateway terminates TLS.

## Host guard and proxies

Loopback binds accept `localhost`, `127.0.0.1`, and `[::1]` Host values by
default, with or without the listening port, and reject unrelated values. A
non-loopback bind fails before opening its listener unless you provide an
explicit Host policy:

```bash
biomcp serve-http --host 0.0.0.0 --port 8000 \
  --allowed-hosts biomcp.example.org,localhost:8000
```

If a proxy rewrites Host headers, include the value BioMCP actually receives.
BioMCP does not infer trust from `Forwarded` or `X-Forwarded-Host` headers.
Each allowlist entry is an exact hostname or IP address with an optional port;
write an IPv6 address with a port as `[::1]:8000`. Entries are normalized for
case, trailing dots, IP spelling, and duplicates before BioMCP listens. Empty
entries, schemes, paths, wildcards, internal whitespace, malformed addresses,
and ports outside 1–65535 are startup errors.
The policy covers `/mcp`, `/`, `/health`, and `/readyz`; probe routes are not an
exception to the DNS-rebinding boundary.

`--unsafe-allow-any-host` is an explicit escape hatch for infrastructure that
must accept arbitrary Host values. It disables only the Host check and is
mutually exclusive with `--allowed-hosts`; it adds no authentication or
encryption.

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

The MCP server advertises both tools and resources. Every BioMCP MCP tool is annotated
`readOnlyHint: true`, has a human-friendly title, and has a non-empty description so
clients and directories can present the surface as read-only. The MCP command allowlist
still enforces that read-only boundary when a tool is called.

### Typed tools

Prefer the typed tools when possible:

- `search` for biomedical searches across supported entity types.
- `get` for record lookup and sectioned detail retrieval.
- `variant_normalize_car` for bounded ClinGen Allele Registry normalization.
- `variant_erepo` for bounded ClinGen ERepo assertions.
- `gene_cspec` for bounded ClinGen CSpec manifests and pages.
- `variant_articles` for compact multi-variant literature shortlists.

Their schemas enumerate valid entity names, valid get section tokens, and the bounded search `limit`.

### Raw command escape hatch

The raw `biomcp` tool remains available for read-only CLI commands outside the first typed slice. It is an escape hatch, not the preferred first call. It accepts read-only commands such as `discover`, `biomcp skill list`, `biomcp skill render`, embedded `biomcp skill <number-or-slug>` lookups, and the catalog-only `study download --list` form.

The seven-tool catalog is intentionally bounded. Reproduce its current local
measurement with `uv run --no-sync python scripts/measure-mcp-tools.py`. CI
rejects the full catalog above 22,600 UTF-8 bytes or 5,800 `cl100k_base` tokens,
and rejects the raw `biomcp` description above 4,000 bytes. The 22,600-byte /
5,800-token CI budget applies to the 0.9.0-dev.5 development build. The
aggregate ceilings combine its current 15,841-byte, 3,996-token catalog with
the largest typed entry (`search`), leaving 99 bytes and 52 tokens of margin.
Exact current counts belong to that executable measurement rather than
hand-copied documentation.

Mutating or workstation-local commands are blocked in MCP mode. Examples include `skill install`, `skill status`, local source sync commands, `update`, `uninstall`, and `study download <study_id>`. Status remains CLI-only because probing an arbitrary skill directory can reveal workstation-local paths. Cache-family commands such as `cache path`, `cache stats`, `cache clean`, and `cache clear` are also rejected because they reveal workstation-local paths and filesystem context.

### Resources

Current builds always publish the help resource and one markdown resource per embedded skill use-case.

| URI | Name | Notes |
|---|---|---|
| `biomcp://help` | BioMCP Overview | Same in-memory help text returned by `biomcp skill render`. |
| `biomcp://skill/<slug>` | Embedded worked example | Markdown resource for a built-in BioMCP skill use-case. |

Workflow playbooks do not add MCP resources. MCP callers that execute BioMCP commands with `--json`, or pass tool input `json: true`, receive the same CLI JSON contract, so responses can include `_meta.workflow`, `_meta.workflow_rationale`, and `_meta.workflow_playbook`. `_meta.next_commands` remains the dynamic current-result follow-up list; unrelated executable examples are never placed in runtime metadata.

## Tool responses

By default, the `biomcp` tool keeps non-chart calls as readable text and appends compact provenance when CLI JSON exposes it:

- `## Sources` rolls up `_meta.section_sources` per section.
- `## Next commands` rolls up `_meta.next_commands` as copyable follow-up commands.

Agents that need the full structured contract can pass the tool input field `json: true`; BioMCP injects `--json` and returns JSON text with metadata such as `_meta.section_sources`, `_meta.evidence_urls`, `_meta.next_commands`, and `_meta.workflow_playbook`.

MCP responses never disclose workstation-local article full-text paths. When full text is saved, readable MCP output reports that it is available while withholding the cache path; JSON replaces the CLI-only `full_text_path` field with `full_text_available: true`. Source, manifest, status, and provenance fields remain available. This transport boundary applies to both local stdio and remote Streamable HTTP MCP clients. Direct CLI calls intentionally continue to print `Saved to:` and serialize `full_text_path` because the CLI user is operating on the machine that owns the file.

In MCP mode, charted `study` commands return two success content blocks in order:

1. `text` with the normal markdown/table output.
2. `image` with `mimeType = "image/svg+xml"` and base64-encoded SVG data.

MCP chart calls do not write files. If the caller supplies `--output` or `-o`, the tool returns an error instructing the caller to consume the inline image instead.

## Operational checklist

- Use `biomcp serve` for local stdio clients.
- Use `biomcp serve-http` for remote Streamable HTTP clients and route them to `/mcp`.
- Probe `/health`, `/readyz`, or `/` from load balancers and deployment checks.
- Keep loopback defaults, or set `--allowed-hosts` for every non-loopback bind.
- Put remote HTTP deployments behind your own authentication and TLS layer.
- Set provider keys as environment variables for the BioMCP process.
