# Installation

This page covers supported BioMCP installation paths and verification checks.

After installation, the `biomcp` command should be available in your shell.

## Option 1: Installer script

```bash
curl -fsSL https://biomcp.org/install.sh | bash
```

The installer downloads a prebuilt binary for your platform (Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64), verifies the SHA-256 checksum, smokes a destination-directory staging file, and atomically places `biomcp` in `~/.local/bin`. It records standalone ownership in adjacent `biomcp.install.json` so update and uninstall cannot damage package-managed installs. It fails closed before replacement when verification fails, and a pending receipt makes interruption recovery deterministic.

The installer never edits shell startup files. If `~/.local/bin` is missing from
`PATH`, it prints one `export PATH=...` command for you to copy. Install
`sha256sum`, `shasum -a 256`, or `openssl dgst -sha256` before running it.

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
- Checksum verification fails: retry the download; the installer intentionally refuses to install an archive without a valid checksum and a local `sha256sum`, `shasum`, or `openssl` SHA-256 tool.
- Normal source builds do not run or require `protoc`; they consume committed
  AlphaGenome generated Rust. Maintainers regenerating that source need pinned
  `protoc` 28.3 and can verify it without writing with
  `scripts/regenerate-alphagenome-proto --check`.
- Network-related health failures: retry and inspect upstream API status.
