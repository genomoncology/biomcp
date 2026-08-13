#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-study-download-error-env"

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "study-download-error" "BIOMCP_STUDY_DOWNLOAD_ERROR"
rm -f "$env_file"
