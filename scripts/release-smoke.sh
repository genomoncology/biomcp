#!/usr/bin/env bash
set -uo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/release-smoke.sh [--bin PATH]

Runs the v0.8.24 release-readiness smoke against the release BioMCP binary.
This is an operator/release check with live calls; it is not part of make spec.

Options:
  --bin PATH   BioMCP binary to test. Defaults to BIOMCP_BIN or target/release/biomcp.
  -h, --help   Show this help.
USAGE
}

current_head_sha() {
  git rev-parse --short=8 HEAD
}

binary_version_output() {
  local bin="$1"
  local output
  output=$("$bin" version 2>/dev/null || true)
  if [[ -z "$(binary_git_sha "$output")" ]]; then
    output=$("$bin" --version 2>/dev/null || true)
  fi
  printf '%s\n' "$output"
}

binary_git_sha() {
  local version_output="$1"
  sed -nE 's/.*\(git ([^,)]*), build .*/\1/p' <<<"$version_output"
}

binary_build_date() {
  local version_output="$1"
  sed -nE 's/.*\(git [^,)]*, build ([^)]*)\).*/\1/p' <<<"$version_output"
}

refresh_binary_metadata() {
  local version_output
  version_output=$(binary_version_output "$BIN")
  BINARY_GIT_SHA=$(binary_git_sha "$version_output")
  BINARY_BUILD_DATE=$(binary_build_date "$version_output")
  [[ -n "$BINARY_GIT_SHA" ]] || BINARY_GIT_SHA="unknown"
  [[ -n "$BINARY_BUILD_DATE" ]] || BINARY_BUILD_DATE="unknown"
}

build_default_release_binary() {
  tools/with-build-identity cargo build --release --locked || exit 2
  BIN="target/release/biomcp"
}

ensure_binary_ready() {
  HEAD_SHA=$(current_head_sha)
  if [[ ! -x "$BIN" ]]; then
    if [[ "$BIN_SOURCE" == "default" ]]; then
      echo "BioMCP release binary not found at $BIN; building target/release/biomcp" >&2
      build_default_release_binary
    else
      echo "BioMCP release binary not found or not executable at $BIN (from $BIN_SOURCE)" >&2
      exit 2
    fi
  fi

  refresh_binary_metadata
  if [[ "$BIN_SOURCE" == "default" && "$BINARY_GIT_SHA" != "$HEAD_SHA" ]]; then
    # The default release smoke must never test a stale target/release binary.
    echo "BioMCP release binary at $BIN is stamped git $BINARY_GIT_SHA, not HEAD $HEAD_SHA; rebuilding" >&2
    build_default_release_binary
    refresh_binary_metadata
    if [[ "$BINARY_GIT_SHA" != "$HEAD_SHA" ]]; then
      echo "BioMCP release binary at $BIN is still stamped git $BINARY_GIT_SHA after rebuild; expected HEAD $HEAD_SHA" >&2
      exit 2
    fi
  fi
}

BIN_SOURCE="default"
BIN="target/release/biomcp"
HEAD_SHA=""
BINARY_GIT_SHA="unknown"
BINARY_BUILD_DATE="unknown"
REPORT_PATH="target/release-readiness-0.8.24.md"
if [[ -n "${BIOMCP_BIN:-}" ]]; then
  BIN="$BIOMCP_BIN"
  BIN_SOURCE="BIOMCP_BIN"
fi
while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin)
      [[ $# -ge 2 ]] || { echo "--bin requires a path" >&2; exit 2; }
      BIN="$2"
      BIN_SOURCE="--bin"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

ensure_binary_ready
mkdir -p "$(dirname "$REPORT_PATH")"
: >"$REPORT_PATH"
exec > >(tee "$REPORT_PATH")

PASS=0
FAIL=0
FAIL_ITEMS=()
TMP_ROOT=$(mktemp -d)
SERVER_PID=""
trap '[[ -n "${SERVER_PID:-}" ]] && kill "$SERVER_PID" 2>/dev/null || true; rm -rf "$TMP_ROOT"' EXIT

record_pass() {
  local name="$1"
  echo "PASS $name"
  PASS=$((PASS + 1))
}

record_fail() {
  local name="$1"
  local detail="$2"
  echo "FAIL $name — $detail"
  FAIL=$((FAIL + 1))
  FAIL_ITEMS+=("$name — $detail")
}

assert_cmd() {
  local name="$1"
  local timeout_seconds="$2"
  local expected_code="$3"
  local stdout_re="$4"
  local stderr_re="$5"
  shift 5

  local out="$TMP_ROOT/${name//[^A-Za-z0-9_.-]/_}.out"
  local err="$TMP_ROOT/${name//[^A-Za-z0-9_.-]/_}.err"
  timeout "$timeout_seconds" "$BIN" "$@" >"$out" 2>"$err"
  local code=$?

  if [[ "$code" != "$expected_code" ]]; then
    record_fail "$name" "exit $code, expected $expected_code; stderr: $(head -n 2 "$err" | tr '\n' ' ')"
    return
  fi
  if [[ -n "$stdout_re" ]] && ! grep -qE -- "$stdout_re" "$out"; then
    record_fail "$name" "stdout did not match /$stdout_re/"
    return
  fi
  if [[ -n "$stderr_re" ]] && ! grep -qE -- "$stderr_re" "$err"; then
    record_fail "$name" "stderr did not match /$stderr_re/"
    return
  fi
  record_pass "$name"
}

assert_no_stderr_warn() {
  local name="$1"
  local timeout_seconds="$2"
  shift 2
  local out="$TMP_ROOT/${name//[^A-Za-z0-9_.-]/_}.out"
  local err="$TMP_ROOT/${name//[^A-Za-z0-9_.-]/_}.err"
  timeout "$timeout_seconds" "$BIN" "$@" >"$out" 2>"$err"
  local code=$?
  if [[ "$code" != 0 ]]; then
    record_fail "$name" "exit $code, expected 0; stderr: $(head -n 2 "$err" | tr '\n' ' ')"
    return
  fi
  if grep -q 'WARN' "$err"; then
    record_fail "$name" "stderr contained WARN: $(grep 'WARN' "$err" | head -n 1)"
    return
  fi
  record_pass "$name"
}

free_port() {
  python3 - <<'PY'
import socket
s = socket.socket()
s.bind(('127.0.0.1', 0))
print(s.getsockname()[1])
s.close()
PY
}

wait_for_http() {
  local url="$1"
  for _ in {1..40}; do
    if curl -fsS --max-time 2 "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.25
  done
  return 1
}

smoke_serve_http() {
  local port
  port=$(free_port)
  "$BIN" serve-http --host 127.0.0.1 --port "$port" >"$TMP_ROOT/serve-http-open.out" 2>"$TMP_ROOT/serve-http-open.err" &
  SERVER_PID=$!
  if ! wait_for_http "http://127.0.0.1:$port/health"; then
    record_fail "240 serve-http default Host guard" "server did not become healthy"
  else
    local code
    code=$(curl -sS --max-time 5 -H 'Host: evil.example' -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port/health" || true)
    if [[ "$code" == "200" ]]; then
      record_pass "240 serve-http default accepts foreign Host"
    else
      record_fail "240 serve-http default accepts foreign Host" "HTTP $code, expected 200"
    fi
  fi
  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""

  port=$(free_port)
  "$BIN" serve-http --host 127.0.0.1 --port "$port" --allowed-hosts allowed.example >"$TMP_ROOT/serve-http-locked.out" 2>"$TMP_ROOT/serve-http-locked.err" &
  SERVER_PID=$!
  if ! wait_for_http "http://127.0.0.1:$port/health"; then
    record_fail "240 serve-http allowed-hosts rejects foreign Host" "server did not become healthy"
  else
    local code
    code=$(curl -sS --max-time 5 -H 'Host: evil.example' -o /dev/null -w '%{http_code}' "http://127.0.0.1:$port/mcp" || true)
    if [[ "$code" == "403" ]]; then
      record_pass "240 serve-http allowed-hosts rejects foreign Host"
    else
      record_fail "240 serve-http allowed-hosts rejects foreign Host" "HTTP $code, expected 403 from /mcp"
    fi
  fi
  kill "$SERVER_PID" 2>/dev/null || true
  wait "$SERVER_PID" 2>/dev/null || true
  SERVER_PID=""
}

smoke_plugin() {
  if jq -e '.plugins[] | select(.mcpServers.biomcp.command == "biomcp" and (.mcpServers.biomcp.args == ["serve"]))' .claude-plugin/marketplace.json >/dev/null; then
    record_pass "239 plugin marketplace wires biomcp serve"
  else
    record_fail "239 plugin marketplace wires biomcp serve" "marketplace.json missing biomcp serve wiring"
  fi
}

smoke_mcp() {
  local mcp_out="$TMP_ROOT/mcp.tsv"
  BIOMCP_BIN="$BIN" python3 - <<'PY' >"$mcp_out"
import json, os, subprocess, sys
bin_path = os.environ['BIOMCP_BIN']
p = subprocess.Popen([bin_path, 'serve'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, bufsize=1)

def request(msg):
    p.stdin.write(json.dumps(msg) + '\n')
    p.stdin.flush()
    line = p.stdout.readline()
    if not line:
        raise RuntimeError('MCP server closed stdout')
    return json.loads(line)

try:
    request({'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'release-smoke','version':'0'}}})
    p.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized'}) + '\n')
    p.stdin.flush()
    tools = request({'jsonrpc':'2.0','id':2,'method':'tools/list','params':{}})['result']['tools']
    by_name = {tool['name']: tool for tool in tools}
    expected = ['biomcp', 'search', 'get', 'variant_normalize_car', 'variant_erepo', 'gene_cspec', 'variant_articles']
    typed_ok = [tool['name'] for tool in tools] == expected
    typed_ok = typed_ok and 'enum' in by_name['search']['inputSchema']['properties']['entity']
    typed_ok = typed_ok and 'enum' in by_name['get']['inputSchema']['properties']['entity']
    print('435 typed MCP tool surface\t' + ('PASS' if typed_ok else 'FAIL') + '\tsearch/get tools with entity enum schemas')

    call = request({'jsonrpc':'2.0','id':3,'method':'tools/call','params':{'name':'get','arguments':{'entity':'gene','id':'BRAF','json':False}}})
    text = '\n'.join(item.get('text','') for item in call.get('result',{}).get('content',[]) if item.get('type') == 'text')
    provenance_ok = '## Sources' in text and 'Identity:' in text and 'NCBI Gene' in text
    print('434 MCP provenance footer\t' + ('PASS' if provenance_ok else 'FAIL') + '\tget gene BRAF includes section sources')
finally:
    p.kill()
PY
  local name status detail
  while IFS=$'\t' read -r name status detail; do
    [[ -n "$name" ]] || continue
    if [[ "$status" == "PASS" ]]; then
      record_pass "$name"
    else
      record_fail "$name" "$detail"
    fi
  done <"$mcp_out"
}

smoke_version() {
  local short_sha
  short_sha=$(git rev-parse --short=8 HEAD)
  assert_cmd "444 --version reports 0.8.24" 10 0 '0\.8\.24' '' --version
  assert_cmd "444 version command reports HEAD git SHA" 10 0 "git ${short_sha}" '' version
}

echo "== BioMCP v0.8.24 release smoke =="
echo "Binary: $BIN"
echo "Binary git SHA: $BINARY_GIT_SHA"
echo "Binary build date: $BINARY_BUILD_DATE"
echo "Current HEAD: $HEAD_SHA"
if [[ "$BIN_SOURCE" != "default" && "$BINARY_GIT_SHA" != "$HEAD_SHA" ]]; then
  echo "WARNING: $BIN_SOURCE binary is stamped git $BINARY_GIT_SHA, not current HEAD $HEAD_SHA"
fi

smoke_serve_http
smoke_plugin
smoke_mcp

assert_cmd "436 limit over cap states range" 30 2 '' '--limit must be between 1 and 50' search gene BRAF --limit 51
assert_cmd "436 multi-word get drug guidance" 60 1 '' 'Try searching: biomcp search drug -q "foo bar"' get drug 'foo bar'
assert_cmd "436 multi-word get disease guidance" 60 1 '' 'Try searching: biomcp search disease -q "foo bar"' get disease 'foo bar'
assert_cmd "436 get pathway UniProt redirect" 60 2 '' 'did you mean `biomcp get protein P15056`' get pathway P15056

assert_no_stderr_warn "437 normal query emits no WARN" 90 get gene BRAF

assert_cmd "438 gene trials --limit 1 exits promptly" 45 0 'Results: 1' '' gene trials BRAF --limit 1
assert_cmd "438 disease trials --limit 1 exits promptly" 45 0 'Results: 1' '' disease trials melanoma --limit 1

assert_cmd "439 alias PD-L1 resolves CD274" 90 0 '^# CD274 ' '' get gene PD-L1
assert_cmd "439 alias HER2 resolves ERBB2" 90 0 '^# ERBB2 ' '' get gene HER2
assert_cmd "439 alias P53 resolves TP53" 90 0 '^# TP53 ' '' get gene P53

assert_cmd "440 transcript HGVS variant resolves" 180 0 'BRAF p\.V600E|rs113488022' '' get variant 'NM_004333.6:c.1799T>A'

assert_cmd "441 JSON unknown gene error on stdout" 45 1 '"error"[[:space:]]*:' '' --json get gene NOT_A_GENE_445
assert_cmd "441 JSON bogus variant error on stdout" 45 2 '"error"[[:space:]]*:' '' --json get variant bogusvar445

assert_cmd "443 pathway Ensembl redirect" 60 2 '' 'did you mean `biomcp get gene ENSG00000157764`' get pathway ENSG00000157764
assert_cmd "443 pathway symbol redirect" 60 2 '' 'did you mean `biomcp get gene BRAF`' get pathway BRAF
assert_cmd "443 pathway rsID redirect" 60 2 '' 'did you mean `biomcp get variant rs113488022`' get pathway rs113488022

smoke_version

echo
echo "== release smoke summary =="
echo "PASS: $PASS"
echo "FAIL: $FAIL"
if [[ "$FAIL" -gt 0 ]]; then
  echo "Failures:"
  printf ' - %s\n' "${FAIL_ITEMS[@]}"
  exit 1
fi
