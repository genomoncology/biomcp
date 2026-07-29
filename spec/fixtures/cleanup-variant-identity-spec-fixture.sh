#!/usr/bin/env bash
set -euo pipefail
workspace_root="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "variant-identity" "$workspace_root/.cache/spec-variant-identity-env" "BIOMCP_VARIANT_IDENTITY"
