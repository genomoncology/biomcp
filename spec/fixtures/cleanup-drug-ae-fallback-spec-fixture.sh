#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-drug-ae-fallback-env"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "drug-ae-fallback" "BIOMCP_DRUG_AE_FALLBACK"
rm -f "$env_file"
