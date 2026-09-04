# Add BioMCP to your MCP client

Use this page after installing BioMCP to connect the local `biomcp serve` stdio
server to your MCP client.

!!! warning "Install `biomcp-cli`, not `biomcp`"
    The PyPI package for this project is `biomcp-cli`. It installs the
    `biomcp` binary used below. Do **not** run `pip install biomcp`; that PyPI
    package is unrelated to this project.

    ```bash
    uv tool install biomcp-cli
    biomcp --version
    ```

## Fast path: generate the client config

BioMCP can print copy-paste setup for the supported local stdio clients:

```bash
biomcp mcp-config --client codex
biomcp mcp-config --client claude-desktop
biomcp mcp-config --client claude-code
biomcp mcp-config --client cursor
biomcp mcp-config --client cline
biomcp mcp-config --client vscode
biomcp mcp-config --client json
```

If your MCP client cannot see the same shell `PATH` where `biomcp` is
installed, add `--absolute-path`:

```bash
biomcp mcp-config --client claude-desktop --absolute-path
```

The sections below are static fallbacks you can copy directly.

## Choose your client

- [Codex](#codex)
- [Claude Code](#claude-code)
- [Claude Desktop](#claude-desktop)
- [Cursor](#cursor)
- [Cline](#cline)
- [VS Code](#vs-code)
- [Generic MCP JSON](#generic-mcp-json)

## Codex

Generate the command:

```bash
biomcp mcp-config --client codex
```

Or add the server directly:

```bash
codex mcp add biomcp -- biomcp serve
```

## Claude Code

For Claude Code, prefer the BioMCP plugin marketplace. Install the `biomcp`
binary first, then run these Claude Code slash commands:

```text
/plugin marketplace add genomoncology/biomcp
/plugin install biomcp@biomcp
```

The plugin wires Claude Code to the local stdio MCP server with `biomcp serve`.
If you need manual JSON instead, generate it with:

```bash
biomcp mcp-config --client claude-code
```

## Claude Desktop

When available in your Claude Desktop build, install BioMCP from the Anthropic
Directory or MCPB extension flow. For a local manual setup, generate the JSON:

```bash
biomcp mcp-config --client claude-desktop
```

Then add this server entry to your Claude Desktop config:

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

Restart Claude Desktop after changing the config.

## Cursor

Generate the Cursor config:

```bash
biomcp mcp-config --client cursor
```

Or add this to your Cursor MCP config:

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

## Cline

Generate the Cline config:

```bash
biomcp mcp-config --client cline
```

Or add this server to Cline's MCP settings:

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

## VS Code

Generate the VS Code config:

```bash
biomcp mcp-config --client vscode
```

Or add this server to `.vscode/mcp.json` in your workspace, or to the file VS Code opens with **MCP: Open User Configuration**. VS Code reads `servers` here, not the `mcpServers` key the other clients use — a snippet under the wrong key is valid JSON and is then silently ignored, so the server never shows up.

```json
{
  "servers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["serve"]
    }
  }
}
```

## Generic MCP JSON

Use the generic JSON form for any stdio MCP client that accepts an `mcpServers`
object:

```bash
biomcp mcp-config --client json
```

Static fallback:

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

## Validate before connecting

Run quick local checks before restarting your client:

```bash
biomcp --version
biomcp health --apis-only
```

If the client still cannot start BioMCP, regenerate its config with
`--absolute-path` and paste the result into the client settings.

## Related docs

- [Installation](installation.md)
- [API keys](api-keys.md)
- [MCP server reference](../reference/mcp-server.md)
- [Remote HTTP Server](remote-http.md)
