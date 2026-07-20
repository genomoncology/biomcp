#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-variant-identity-env"

if [[ -f "$env_file" ]]; then
  # shellcheck source=/dev/null
  . "$env_file"
fi

if [[ -n "${BIOMCP_VARIANT_IDENTITY_PID:-}" ]] && kill -0 "$BIOMCP_VARIANT_IDENTITY_PID" 2>/dev/null; then
  kill "$BIOMCP_VARIANT_IDENTITY_PID" 2>/dev/null || true
  for _ in $(seq 1 50); do
    if ! kill -0 "$BIOMCP_VARIANT_IDENTITY_PID" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done
  if kill -0 "$BIOMCP_VARIANT_IDENTITY_PID" 2>/dev/null; then
    kill -KILL "$BIOMCP_VARIANT_IDENTITY_PID" 2>/dev/null || true
  fi
fi

if [[ -n "${BIOMCP_VARIANT_IDENTITY_ROOT:-}" ]]; then
  rm -rf "$BIOMCP_VARIANT_IDENTITY_ROOT"
fi

rm -f "$env_file"
