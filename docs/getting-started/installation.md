# Installation

This page covers supported BioMCP installation paths and verification checks.

After installation, the `biomcp` command should be available in your shell.

## Option 1: Installer script

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

The installer downloads a prebuilt binary for your platform (Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64), verifies the SHA256 checksum, and places `biomcp` in `~/.local/bin`.

Pin a specific version:

```bash
curl -fsSL https://biomcp.org/install.sh | bash -s -- --version 0.8.0
```

Verify:

```bash
biomcp --version
```

## Option 2: PyPI package

```bash
uv tool install biomcp-cli
# or, inside an active Python environment:
# pip install biomcp-cli
```

Install the `biomcp-cli` package, then use the `biomcp` command in the rest of
this guide.

Verify:

```bash
biomcp --version
```

## Homebrew

```bash
brew tap genomoncology/biomcp
brew install biomcp
```

The separate `genomoncology/homebrew-biomcp` tap repository must exist before
these commands can work; creating that tap is a one-time release prerequisite.

Verify:

```bash
biomcp --version
```

## Option 3: Source build

From a local checkout:

```bash
make install
"$HOME/.local/bin/biomcp" --version
```

## Option 4: Docker image

Use the published GHCR image when you want BioMCP without a local Rust or Python toolchain:

```bash
docker run --rm ghcr.io/genomoncology/biomcp --version
docker run --rm ghcr.io/genomoncology/biomcp list
```

For stdio MCP clients, run the same image with `serve` and keep stdin open:

```bash
docker run --rm -i ghcr.io/genomoncology/biomcp serve
```

Pass provider keys from your shell when needed, for example `-e ONCOKB_TOKEN` or `-e NCBI_API_KEY`. Do not put secret values in documentation or checked-in client configs.

## Post-install smoke checks

```bash
biomcp list
biomcp health --apis-only
biomcp search gene -q BRAF --limit 1
```

## Environment notes

- Default output is markdown.
- Use `--json` when a workflow needs structured output.
- Add BioMCP to Codex, Claude Code, Claude Desktop, Cursor, Cline, VS Code, or another MCP client with the [MCP clients guide](mcp-clients.md).
- Optional API keys are documented in [API keys](api-keys.md).

## Troubleshooting quick hits

- Command not found: ensure install location is on `PATH`.
- Build fails at protobuf step: install `protoc`.
- Network-related health failures: retry and inspect upstream API status.
