#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-../..}"
repo_root="$(git -C "$workspace_root" rev-parse --show-toplevel 2>/dev/null || printf '%s\n' "$workspace_root")"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

env_file="$repo_root/.cache/spec-article-fulltext-source-env"
if [[ ! -f "$env_file" ]]; then
  bash "$script_dir/setup-article-fulltext-source-fixture.sh" "$repo_root" >/dev/null
  trap 'bash "$script_dir/cleanup-article-fulltext-source-fixture.sh" "$repo_root"' EXIT
fi
# shellcheck disable=SC1091
. "$env_file"

cache_dir="$repo_root/.cache/spec-article-fulltext-jats-cache"
rm -rf "$cache_dir"
export BIOMCP_CACHE_DIR="$cache_dir"
export BIOMCP_CACHE_MODE="off"

binary="${BIOMCP_BIN:-$repo_root/target/release/biomcp}"
out="$($binary get article 22663011 fulltext)"
saved_path="$(printf '%s\n' "$out" | sed -n 's/^Saved to: //p' | head -n1)"

if [ -z "$saved_path" ]; then
  printf '%s\n' "$out" >&2
  exit 1
fi

cat "$saved_path"
