#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${QUALITY_RATCHET_OUTPUT_DIR:-$ROOT_DIR/.march/reality-check}"
SPEC_GLOB="${QUALITY_RATCHET_SPEC_GLOB:-$ROOT_DIR/spec/**/*.md}"
CLI_FILE="${QUALITY_RATCHET_CLI_FILE:-$ROOT_DIR/src/cli/mod.rs}"
SHELL_FILE="${QUALITY_RATCHET_SHELL_FILE:-$ROOT_DIR/src/mcp/shell.rs}"
CATALOG_FILE="${QUALITY_RATCHET_CATALOG_FILE:-$ROOT_DIR/src/mcp/catalog.rs}"
SOURCES_DIR="${QUALITY_RATCHET_SOURCES_DIR:-$ROOT_DIR/src/sources}"
SOURCES_MOD="${QUALITY_RATCHET_SOURCES_MOD:-$ROOT_DIR/src/sources/mod.rs}"
HEALTH_FILE="${QUALITY_RATCHET_HEALTH_FILE:-$ROOT_DIR/src/cli/health/catalog.rs}"
CLI_LINE_CAP_ALLOWLIST="${QUALITY_RATCHET_CLI_LINE_CAP_ALLOWLIST:-$ROOT_DIR/tools/cli-line-cap-allowlist.json}"

audit_args=()
if [[ -n "${QUALITY_RATCHET_AUDITS:-}" ]]; then
  read -r -a requested_audits <<<"$QUALITY_RATCHET_AUDITS"
  for audit in "${requested_audits[@]}"; do
    audit_args+=(--audit "$audit")
  done
fi

mkdir -p "$OUTPUT_DIR"

# Continuous integration passes QUALITY_RATCHET_REQUIRE_LOCAL_NAMES=1 so a
# missing local forbidden-name declaration fails loudly instead of letting
# the tree scan pass with inert example placeholders only (ticket 2038
# finding 7, 2035 #11). Local runs print a note and keep going.
zero_coupling_args=()
if [[ -n "${QUALITY_RATCHET_REQUIRE_LOCAL_NAMES:-}" ]]; then
  zero_coupling_args+=("--require-local-names")
fi

uv run --no-project python "$ROOT_DIR/tools/check-zero-coupling.py" \
  --root "$ROOT_DIR" "${zero_coupling_args[@]+${zero_coupling_args[*]}}"

exec uv run --no-project python "$ROOT_DIR/tools/check-quality-ratchet.py" \
  --root-dir "$ROOT_DIR" \
  --output-dir "$OUTPUT_DIR" \
  --spec-glob "$SPEC_GLOB" \
  --cli-file "$CLI_FILE" \
  --shell-file "$SHELL_FILE" \
  --catalog-file "$CATALOG_FILE" \
  --sources-dir "$SOURCES_DIR" \
  --sources-mod "$SOURCES_MOD" \
  --health-file "$HEALTH_FILE" \
  --cli-line-cap-allowlist "$CLI_LINE_CAP_ALLOWLIST" \
  "${audit_args[@]}"
