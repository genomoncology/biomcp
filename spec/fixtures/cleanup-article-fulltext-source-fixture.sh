#!/usr/bin/env bash
set -euo pipefail
workspace_root="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
bash "$script_dir/routine-fixture-ownership.sh" cleanup "$workspace_root" "article-fulltext-source" "$workspace_root/.cache/spec-article-fulltext-source-env" "BIOMCP_ARTICLE_FULLTEXT_SOURCE_FIXTURE"
