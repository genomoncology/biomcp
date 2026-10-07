#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:?repo root required}"
ROOT="$(cd "$ROOT" && pwd)"
# Optional arguments: the forced deadline in milliseconds (default 8000)
# and, after it, search-article arguments. With no arguments the invocation
# stays byte-identical to the recorded federated case.
DEADLINE_MS="${2:-8000}"
if (( $# > 1 )); then
  shift 2
else
  shift "$#"
fi
cleanup() {
  if [[ -n "${holder_pid:-}" ]]; then
    kill "$holder_pid" 2>/dev/null || true
    wait "$holder_pid" 2>/dev/null || true
  fi
  bash "$ROOT/spec/fixtures/cleanup-article-search-deadline-fixture.sh" "$ROOT"
}
trap cleanup EXIT

bash "$ROOT/spec/fixtures/setup-article-search-deadline-fixture.sh" "$ROOT"
# shellcheck source=/dev/null
. "$ROOT/.cache/spec-article-search-deadline-env"
export BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS="$DEADLINE_MS"

# HOLD_EPOCH_LOCK=1 holds the cache epoch lock from a side process for the
# whole invocation, pinning that construction cannot outlive the deadline
# (ticket 1299).
holder_pid=""
if [[ "${HOLD_EPOCH_LOCK:-0}" == "1" ]]; then
  mkdir -p "$BIOMCP_CACHE_DIR"
  holder_marker="$(mktemp -u "$BIOMCP_CACHE_DIR/.holder.XXXXXX")"
  python3 -c '
import fcntl, sys, time
lock_path, marker_path = sys.argv[1], sys.argv[2]
lock = open(lock_path, "a+")
fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
with open(marker_path, "w") as marker:
    marker.write("held")
time.sleep(60)
' "$BIOMCP_CACHE_DIR/.body-limit-cache-v1.lock" "$holder_marker" &
  holder_pid=$!
  for _ in $(seq 1 50); do
    [[ -f "$holder_marker" ]] && break
    kill -0 "$holder_pid" 2>/dev/null || break
    sleep 0.05
  done
  test -f "$holder_marker"
fi

if (( $# )); then
  timeout 30s "$ROOT/tools/biomcp-ci" --json search article "$@"
else
  timeout 25s "$ROOT/tools/biomcp-ci" --json search article -k "deadline-bound federation" --full --limit 3
fi
