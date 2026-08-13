#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup \
  "$workspace_root" "article-federated-timeout" \
  "BIOMCP_ARTICLE_FEDERATED_TIMEOUT_FIXTURE"
rm -f "$workspace_root/.cache/spec-article-federated-timeout-env"
