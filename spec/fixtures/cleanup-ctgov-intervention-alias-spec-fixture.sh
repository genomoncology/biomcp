#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
env_file="$workspace_root/.cache/spec-ctgov-intervention-alias-env"

if [ -f "$env_file" ]; then
  # shellcheck disable=SC1090
  . "$env_file"
fi

if [ -n "${BIOMCP_CTGOV_INTERVENTION_ALIAS_PID:-}" ]; then
  kill "$BIOMCP_CTGOV_INTERVENTION_ALIAS_PID" 2>/dev/null || true
fi

if [ -n "${BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT:-}" ]; then
  rm -rf "$BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT"
fi

rm -f "$env_file"
