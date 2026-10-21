#!/usr/bin/env bash
# Full source collection belongs to nightly and release, never routine lint.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec uv run --no-sync python tools/collect_feature_coverage.py "$@"
