#!/usr/bin/env bash
set -euo pipefail
workspace_root="$(cd "${1:-$PWD}" && pwd)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "clingen-cspec" "BIOMCP_CSPEC_FIXTURE"
rm -f "$workspace_root/.cache/spec-clingen-cspec-env"
