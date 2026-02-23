# Installation

This page covers supported BioMCP installation paths and verification checks.

After installation, the `biomcp` command should be available in your shell.

## Option 1: Installer script (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/genomoncology/biomcp/main/install.sh | bash
```

The installer downloads a prebuilt binary for your platform (Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64), verifies the SHA256 checksum, and places `biomcp` in `~/.local/bin`.

Pin a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/genomoncology/biomcp/main/install.sh | bash -s -- --version 0.8.0
```

Verify:

```bash
biomcp --version
```

## Option 2: Source build

From a local checkout:

```bash
cargo build --release --locked
./target/release/biomcp --version
```

Install into Cargo bin path:

```bash
cargo install --path . --locked
biomcp --version
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
