#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-complexportal-env"

if [ ! -f "$env_file" ]; then
  exit 0
fi

set +u
# shellcheck disable=SC1090
. "$env_file"
set -u

pid_matches_fixture() {
  local pid="$1"
  local ready_file="$2"

  [ -r "/proc/$pid/cmdline" ] || return 1
  tr '\0' '\n' <"/proc/$pid/cmdline" | grep -Fqx -- "$ready_file"
}

if [ -n "${BIOMCP_COMPLEXPORTAL_FIXTURE_PID:-}" ] \
  && [ -n "${BIOMCP_COMPLEXPORTAL_FIXTURE_READY_FILE:-}" ] \
  && kill -0 "$BIOMCP_COMPLEXPORTAL_FIXTURE_PID" 2>/dev/null \
  && pid_matches_fixture "$BIOMCP_COMPLEXPORTAL_FIXTURE_PID" "$BIOMCP_COMPLEXPORTAL_FIXTURE_READY_FILE"; then
  kill "$BIOMCP_COMPLEXPORTAL_FIXTURE_PID" 2>/dev/null || true
fi

case "${BIOMCP_COMPLEXPORTAL_FIXTURE_ROOT:-}" in
  "$cache_dir"/spec-complexportal.*)
    rm -rf "$BIOMCP_COMPLEXPORTAL_FIXTURE_ROOT"
    ;;
esac

rm -f "$env_file"
