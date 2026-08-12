#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-drug-ae-fallback-env"

if [ ! -f "$env_file" ]; then
  exit 0
fi

set +u
# The setup script writes this test-owned environment file.
# shellcheck disable=SC1090
. "$env_file"
set -u

pid_matches_fixture() {
  local pid="$1"
  local ready_file="$2"

  [ -r "/proc/$pid/cmdline" ] || return 1
  tr '\0' '\n' <"/proc/$pid/cmdline" | grep -Fqx -- "$ready_file"
}

if [ -n "${BIOMCP_DRUG_AE_FALLBACK_PID:-}" ] \
  && [ -n "${BIOMCP_DRUG_AE_FALLBACK_READY_FILE:-}" ] \
  && kill -0 "$BIOMCP_DRUG_AE_FALLBACK_PID" 2>/dev/null \
  && pid_matches_fixture "$BIOMCP_DRUG_AE_FALLBACK_PID" "$BIOMCP_DRUG_AE_FALLBACK_READY_FILE"; then
  kill "$BIOMCP_DRUG_AE_FALLBACK_PID" 2>/dev/null || true
fi

case "${BIOMCP_DRUG_AE_FALLBACK_ROOT:-}" in
  "$cache_dir"/spec-drug-ae-fallback.*)
    rm -rf "$BIOMCP_DRUG_AE_FALLBACK_ROOT"
    ;;
esac

rm -f "$env_file"
