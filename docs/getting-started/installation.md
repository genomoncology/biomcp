# Installation

This page covers supported BioMCP installation paths and verification checks.

After installation, the `biomcp` command should be available in your shell.

## Option 1: PyPI package

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

## Option 2: Installer script

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

## Option 3: Source build

From a local checkout:

```bash
make install
"$HOME/.local/bin/biomcp" --version
```

## Post-install smoke checks

```bash
biomcp list
biomcp health --apis-only
biomcp search gene -q BRAF --limit 1
```

## Environment notes

- Default output is markdown.
- Use `--json` when a workflow needs structured output.
- Optional API keys are documented in [API keys](api-keys.md).

## Troubleshooting quick hits

- Command not found: ensure install location is on `PATH`.
- Build fails at protobuf step: install `protoc`.
- Network-related health failures: retry and inspect upstream API status.
