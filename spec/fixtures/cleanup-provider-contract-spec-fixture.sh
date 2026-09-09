#!/usr/bin/env bash
set -euo pipefail
workspace_root="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
env_file="$workspace_root/.cache/spec-provider-contract-env"
if [[ -s "$env_file" ]]; then
  # shellcheck source=/dev/null
  source "$env_file"
  candidate="${BIOMCP_GENCC_FIXTURE_PARENT:-}"
  candidate_root="${BIOMCP_GENCC_FIXTURE_TMP_ROOT:-${TMPDIR:-/tmp}}"
  if [[ "$candidate" == "$candidate_root/biomcp-gencc-provider-contract."* && -d "$candidate" && ! -L "$candidate" ]]; then
    rm -rf -- "$candidate"
  fi
fi
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "provider-contract" "BIOMCP_PROVIDER_CONTRACT"
rm -f "$env_file"
