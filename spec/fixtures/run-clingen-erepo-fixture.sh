#!/usr/bin/env bash
# Frozen ERepo fixture entry point. The build step replaces this direct absent-surface
# invocation with captured ERepo summary and SEPIO-detail responses, then projects
# the CLI and typed-MCP observations into the documented stable report.
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"

exec "$binary" --json variant erepo CA015543
