#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
biomcp_bin="${BIOMCP_BIN:-$workspace_root/target/spec/biomcp}"
port="$($workspace_root/spec/fixtures/reserve-local-port)"
log_file="${BIOMCP_SECTION_OUTCOMES_FIXTURE_ROOT:?section-outcome fixture is not running}/mcp.log"
server_pid=""

cleanup() {
  if [ -n "$server_pid" ] && kill -0 "$server_pid" 2>/dev/null; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT

"$biomcp_bin" serve-http --host 127.0.0.1 --port "$port" >"$log_file" 2>&1 8>&- &
server_pid=$!

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$log_file" >&2
    exit 1
  fi
  sleep 0.25
done
curl -fsS "http://127.0.0.1:$port/readyz" >/dev/null

"${BIOMCP_SPEC_MCP_EXAMPLE_BIN:?spec preparation did not export MCP example}" \
  section-outcome "$port"
