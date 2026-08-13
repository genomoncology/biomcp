#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-vaers-env"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "vaers" "BIOMCP_VAERS_FIXTURE"
rm -rf "$cache_dir/spec-vaers-cvx"
rm -f "$env_file"
