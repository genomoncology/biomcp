#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${QUALITY_RATCHET_OUTPUT_DIR:-$ROOT_DIR/.march/reality-check}"
SPEC_GLOB="${QUALITY_RATCHET_SPEC_GLOB:-$ROOT_DIR/spec/**/*.md}"
CLI_FILE="${QUALITY_RATCHET_CLI_FILE:-$ROOT_DIR/src/cli/mod.rs}"
SHELL_FILE="${QUALITY_RATCHET_SHELL_FILE:-$ROOT_DIR/src/mcp/shell.rs}"
BUILD_FILE="${QUALITY_RATCHET_BUILD_FILE:-$ROOT_DIR/build.rs}"
SOURCES_DIR="${QUALITY_RATCHET_SOURCES_DIR:-$ROOT_DIR/src/sources}"
SOURCES_MOD="${QUALITY_RATCHET_SOURCES_MOD:-$ROOT_DIR/src/sources/mod.rs}"
HEALTH_FILE="${QUALITY_RATCHET_HEALTH_FILE:-$ROOT_DIR/src/cli/health.rs}"

mkdir -p "$OUTPUT_DIR"

exec uv run --extra dev python "$ROOT_DIR/tools/check-quality-ratchet.py" \
  --root-dir "$ROOT_DIR" \
  --output-dir "$OUTPUT_DIR" \
  --spec-glob "$SPEC_GLOB" \
  --cli-file "$CLI_FILE" \
  --shell-file "$SHELL_FILE" \
  --build-file "$BUILD_FILE" \
  --sources-dir "$SOURCES_DIR" \
  --sources-mod "$SOURCES_MOD" \
  --health-file "$HEALTH_FILE"
