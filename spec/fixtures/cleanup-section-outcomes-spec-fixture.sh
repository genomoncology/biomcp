#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-section-outcomes-env"

if [ ! -f "$env_file" ]; then
  exit 0
fi

set +u
. "$env_file"
set -u

pid_matches_fixture() {
  local pid="$1"
  local ready_file="$2"
  [ -r "/proc/$pid/cmdline" ] || return 1
  tr '\0' '\n' <"/proc/$pid/cmdline" | grep -Fqx -- "$ready_file"
}

if [ -n "${BIOMCP_SECTION_OUTCOMES_FIXTURE_PID:-}" ] \
  && [ -n "${BIOMCP_SECTION_OUTCOMES_FIXTURE_READY_FILE:-}" ] \
  && kill -0 "$BIOMCP_SECTION_OUTCOMES_FIXTURE_PID" 2>/dev/null \
  && pid_matches_fixture \
    "$BIOMCP_SECTION_OUTCOMES_FIXTURE_PID" \
    "$BIOMCP_SECTION_OUTCOMES_FIXTURE_READY_FILE"; then
  kill "$BIOMCP_SECTION_OUTCOMES_FIXTURE_PID" 2>/dev/null || true
  wait "$BIOMCP_SECTION_OUTCOMES_FIXTURE_PID" 2>/dev/null || true
fi

case "${BIOMCP_SECTION_OUTCOMES_FIXTURE_ROOT:-}" in
  "$cache_dir"/spec-section-outcomes.*)
    rm -rf "$BIOMCP_SECTION_OUTCOMES_FIXTURE_ROOT"
    ;;
esac

rm -f "$env_file"
