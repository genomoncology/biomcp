#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-ctgov-intervention-alias-env"

if [ -f "$env_file" ]; then
  # shellcheck disable=SC1090
  . "$env_file"
fi

fixture_root="${BIOMCP_CTGOV_INTERVENTION_ALIAS_ROOT:-}"
fixture_pgid="${BIOMCP_CTGOV_INTERVENTION_ALIAS_PGID:-}"
server_pid="${BIOMCP_CTGOV_INTERVENTION_ALIAS_SERVER_PID:-}"

root_is_owned=false
case "$fixture_root" in
  "$cache_dir"/spec-ctgov-intervention-alias.*) root_is_owned=true ;;
esac

pgid_is_owned=false
if $root_is_owned && [[ "$fixture_pgid" =~ ^[1-9][0-9]*$ ]]; then
  for pid in "${BIOMCP_CTGOV_INTERVENTION_ALIAS_PID:-}" "$server_pid"; do
    if [[ "$pid" =~ ^[1-9][0-9]*$ ]]; then
      actual_pgid="$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ' || true)"
      if [ "$actual_pgid" = "$fixture_pgid" ]; then
        pgid_is_owned=true
        break
      fi
    fi
  done
fi

if $pgid_is_owned; then
  kill -TERM -- "-$fixture_pgid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    if ! kill -0 -- "-$fixture_pgid" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done
  kill -KILL -- "-$fixture_pgid" 2>/dev/null || true
fi

if $root_is_owned; then
  rm -rf "$fixture_root"
fi
rm -f "$env_file"
