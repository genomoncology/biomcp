#!/usr/bin/env bash
set -euo pipefail

workspace_root="${1:-$PWD}"
workspace_root="$(cd "$workspace_root" && pwd)"
biomcp_bin="${BIOMCP_BIN:-$workspace_root/target/spec/biomcp}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"
cache_dir="$workspace_root/.cache"
kind="run-section-outcome-mcp"
prefix="spec-run-section-outcome-mcp."
mkdir -p "$cache_dir"
recover_fixture_orphans "$cache_dir" "$kind" "$prefix"
fixture_root="$(mktemp -d "$cache_dir/$prefix"XXXXXX)"
owner_arg="$(bash "$ownership_helper" new-owner "$kind" "$fixture_root")"
server_pid_file="$fixture_root/server-pid"
port="$($workspace_root/spec/fixtures/reserve-local-port)"
log_file="$fixture_root/mcp.log"
server_pid=""

cleanup() {
  bash "$ownership_helper" cleanup "$workspace_root" "$kind" "BIOMCP_RUN_SECTION_OUTCOME_MCP"
}
trap cleanup EXIT

prepare_fixture_supervisor_current_process
start_fixture_supervisor "$kind" "$cache_dir" "$fixture_root" "$prefix" "$server_pid_file" \
  bash -c '"$1" serve-http --host 127.0.0.1 --port "$2" & wait $!' "$owner_arg" "$biomcp_bin" "$port" \
  >"$log_file" 2>&1 &
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$server_pid_file" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$server_pid_file"
server_pid="$(<"$server_pid_file")"
bash "$ownership_helper" write "$workspace_root" "$kind" "$fixture_root" "$server_pid" "BIOMCP_RUN_SECTION_OUTCOME_MCP" "$owner_arg" >/dev/null

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
