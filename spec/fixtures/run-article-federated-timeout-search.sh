#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
ROOT="$(cd "$ROOT" && pwd)"
cleanup() {
  bash "$ROOT/spec/fixtures/cleanup-article-federated-timeout-fixture.sh" "$ROOT"
}
trap cleanup EXIT

bash "$ROOT/spec/fixtures/setup-article-federated-timeout-fixture.sh" "$ROOT"
# shellcheck source=/dev/null
. "$ROOT/.cache/spec-article-federated-timeout-env"

timeout 25s "$ROOT/tools/biomcp-ci" search article -k "BRAF melanoma" --source all --debug-plan --limit 3
