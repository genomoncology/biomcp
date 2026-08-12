#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
exec bash "$repo_root/spec/fixtures/run-variant-articles-live-canary.sh" "$repo_root"
