#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
env_file="$workspace_root/.cache/spec-clingen-car-env"
if [[ -f "$env_file" ]]; then
  # shellcheck source=/dev/null
  . "$env_file"
fi
if [[ -n "${BIOMCP_CLINGEN_CAR_PID:-}" ]] && kill -0 "$BIOMCP_CLINGEN_CAR_PID" 2>/dev/null; then
  kill "$BIOMCP_CLINGEN_CAR_PID" 2>/dev/null || true
fi
if [[ -n "${BIOMCP_CLINGEN_CAR_ROOT:-}" ]]; then
  rm -rf "$BIOMCP_CLINGEN_CAR_ROOT"
fi
rm -f "$env_file"
