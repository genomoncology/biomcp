#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "${1:-../..}" && pwd)"
title="$(printf 'Control \007Title')"

exec "$repo_root/tools/biomcp-ci" study query \
  --study msk_impact_2017 \
  --gene TP53 \
  --type mutations \
  --chart bar \
  --title "$title"
