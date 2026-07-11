#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-article-fulltext-source-env"

if [[ -f "$env_file" ]]; then
  # shellcheck source=/dev/null
  . "$env_file"
fi

fixture_root="${BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_ROOT:-}"
fixture_pid="${BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE_PID:-}"
root_is_owned=false
case "$fixture_root" in
  "$cache_dir"/spec-article-fulltext-source.*) root_is_owned=true ;;
esac

pid_is_owned=false
if $root_is_owned && [[ "$fixture_pid" =~ ^[1-9][0-9]*$ ]] \
  && [[ -r "/proc/$fixture_pid/cmdline" ]]; then
  command_line="$(tr '\0' ' ' <"/proc/$fixture_pid/cmdline")"
  if [[ "$command_line" == *"$fixture_root/base-url"* ]] \
    && [[ "$command_line" == *"$fixture_root/request-log.txt"* ]]; then
    pid_is_owned=true
  fi
fi

if $pid_is_owned; then
  kill "$fixture_pid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    if ! kill -0 "$fixture_pid" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done
  kill -KILL "$fixture_pid" 2>/dev/null || true
fi

if $root_is_owned; then
  rm -rf "$fixture_root"
fi
rm -f "$env_file"
