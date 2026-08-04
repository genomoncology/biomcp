#!/usr/bin/env bash
set -euo pipefail
workspace_root="$(cd "${1:-$PWD}" && pwd)"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "cpic" "BIOMCP_CPIC_FIXTURE"
rm -f "$workspace_root/.cache/spec-cpic-env"
