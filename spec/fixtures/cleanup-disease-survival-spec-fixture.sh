#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-disease-survival-env"

if [[ -f "$env_file" ]]; then
  # shellcheck source=/dev/null
  . "$env_file"
fi

if [[ -n "${BIOMCP_DISEASE_SURVIVAL_PID:-}" ]] && kill -0 "$BIOMCP_DISEASE_SURVIVAL_PID" 2>/dev/null; then
  kill "$BIOMCP_DISEASE_SURVIVAL_PID" 2>/dev/null || true
  wait "$BIOMCP_DISEASE_SURVIVAL_PID" 2>/dev/null || true
fi

if [[ -n "${BIOMCP_DISEASE_SURVIVAL_ROOT:-}" ]]; then
  rm -rf "$BIOMCP_DISEASE_SURVIVAL_ROOT"
fi

rm -f "$env_file"
