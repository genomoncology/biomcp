#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
ROOT="$(cd "$ROOT" && pwd)"
cleanup() {
  bash "$ROOT/spec/fixtures/cleanup-article-search-deadline-fixture.sh" "$ROOT"
}
trap cleanup EXIT

bash "$ROOT/spec/fixtures/setup-article-search-deadline-fixture.sh" "$ROOT"
# shellcheck source=/dev/null
. "$ROOT/.cache/spec-article-search-deadline-env"

timeout 25s "$ROOT/tools/biomcp-ci" --json search article -k "deadline-bound federation" --full --limit 3
