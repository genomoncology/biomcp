#!/usr/bin/env bash
set -euo pipefail
root="$(cd "${1:-$PWD}" && pwd)"
env_file="$root/.cache/spec-clingen-cspec-env"
if [[ -f "$env_file" ]]; then
  # shellcheck disable=SC1090
  source "$env_file"
  kill "${BIOMCP_CSPEC_FIXTURE_PID:-}" 2>/dev/null || true
  wait "${BIOMCP_CSPEC_FIXTURE_PID:-}" 2>/dev/null || true
  rm -rf "${BIOMCP_CSPEC_FIXTURE_ROOT:-}"
fi
rm -f "$env_file"
