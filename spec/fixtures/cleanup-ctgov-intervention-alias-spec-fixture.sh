#!/usr/bin/env bash
set -euo pipefail
workspace_root="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "ctgov-intervention-alias" "BIOMCP_CTGOV_INTERVENTION_ALIAS"
rm -f "$workspace_root/.cache/spec-ctgov-intervention-alias-env"
