# Claude Desktop (MCP) Setup

BioMCP can run as an MCP server over stdio. If your Claude Desktop build
offers the Anthropic Directory, install BioMCP there first. Use the JSON config
below when you want a local/manual setup.

## Add BioMCP server config

Use `biomcp serve` as the MCP command:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["serve"]
    }
  }
}
```

If `biomcp` is not on your PATH, use the absolute path to the binary (e.g. `~/.local/bin/biomcp`).

## Validate before connecting Claude

```bash
biomcp --version
biomcp health --apis-only
```

## Verify MCP-level behavior

When connected, clients should discover:

- seven read-only tools: `biomcp`, `search`, `get`, `variant_normalize_car`,
  `variant_erepo`, `gene_cspec`, and `variant_articles`
- one help resource (`biomcp://help`)
- one markdown resource per embedded BioMCP worked example (`biomcp://skill/<slug>`)

Resource discovery gives agent clients both the overview entry point and the
worked-example catalog before execution.

Prefer the bounded typed tools. Use `biomcp` only as the raw read-only escape
hatch, beginning with `biomcp list` for compact command discovery. A current
local `tools/list` measurement is 6,707 UTF-8 bytes and 1,628 `cl100k_base`
tokens; reproduce it with `uv run --no-sync python scripts/measure-mcp-tools.py`.

## Operational tips

- Keep API keys in the client launch environment.
- Restart Claude Desktop after config changes.
- Prefer stable absolute paths in managed environments.

## Related docs

- [Skills](skills.md)
- [API keys](api-keys.md)
- [MCP server reference](../reference/mcp-server.md)
