#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_DIR="${QUALITY_RATCHET_OUTPUT_DIR:-$ROOT_DIR/target/quality-ratchet}"
SPEC_GLOB="${QUALITY_RATCHET_SPEC_GLOB:-$ROOT_DIR/spec/**/*.md}"
CLI_FILE="${QUALITY_RATCHET_CLI_FILE:-$ROOT_DIR/src/cli/mod.rs}"
SHELL_FILE="${QUALITY_RATCHET_SHELL_FILE:-$ROOT_DIR/src/mcp/shell.rs}"
CATALOG_FILE="${QUALITY_RATCHET_CATALOG_FILE:-$ROOT_DIR/src/mcp/catalog.rs}"
SOURCES_DIR="${QUALITY_RATCHET_SOURCES_DIR:-$ROOT_DIR/src/sources}"
SOURCES_MOD="${QUALITY_RATCHET_SOURCES_MOD:-$ROOT_DIR/src/sources/mod.rs}"
HEALTH_FILE="${QUALITY_RATCHET_HEALTH_FILE:-$ROOT_DIR/src/cli/health/catalog.rs}"

audit_args=()
if [[ -n "${QUALITY_RATCHET_AUDITS:-}" ]]; then
  read -r -a requested_audits <<<"$QUALITY_RATCHET_AUDITS"
  for audit in "${requested_audits[@]}"; do
    audit_args+=(--audit "$audit")
  done
fi

mkdir -p "$OUTPUT_DIR"

uv run --no-project python "$ROOT_DIR/tools/check-zero-coupling.py" \
  --root "$ROOT_DIR"

status=0
uv run --no-project python "$ROOT_DIR/tools/check-quality-ratchet.py" \
  --root-dir "$ROOT_DIR" \
  --output-dir "$OUTPUT_DIR" \
  --spec-glob "$SPEC_GLOB" \
  --cli-file "$CLI_FILE" \
  --shell-file "$SHELL_FILE" \
  --catalog-file "$CATALOG_FILE" \
  --sources-dir "$SOURCES_DIR" \
  --sources-mod "$SOURCES_MOD" \
  --health-file "$HEALTH_FILE" \
  "${audit_args[@]}" || status=$?

# The ratchet reports per-audit findings only in its JSON artifacts, so
# a failing run used to exit with no output at all. Print the summary and
# every failing audit's findings so a red gate names its cause.
if [[ "$status" -ne 0 ]]; then
  echo "quality ratchet failed (exit $status); per-audit findings:" >&2
  cat "$OUTPUT_DIR/quality-ratchet-summary.json" >&2 2>/dev/null || true
  for audit_json in "$OUTPUT_DIR"/quality-ratchet-*.json; do
    [[ -f "$audit_json" ]] || continue
    if grep -qE '"status": "(fail|error)"' "$audit_json"; then
      echo "--- $audit_json ---" >&2
      cat "$audit_json" >&2
    fi
  done
fi
exit "$status"
