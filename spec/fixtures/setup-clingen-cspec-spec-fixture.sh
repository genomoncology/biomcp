#!/usr/bin/env bash
# Runner-owned CSpec fixture. Production must retain official IRIs and may rewrite
# only their origin when BIOMCP_CSPEC_FIXTURE_ORIGIN is set by this script.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"

root="$(cd "${1:-$PWD}" && pwd)"
cache="$root/.cache"
env_file="$cache/spec-clingen-cspec-env"
mkdir -p "$cache"
fixture_root="$(mktemp -d "$cache/spec-clingen-cspec.XXXXXX")"
mkdir -p "$fixture_root/cache"
owner_arg="$(bash "$ownership_helper" new-owner "clingen-cspec" "$fixture_root")"
bash "$(dirname "$0")/cleanup-clingen-cspec-spec-fixture.sh" "$root"
ready="$fixture_root/origin"
requests="$fixture_root/requests.jsonl"
: >"$requests"

CAPTURES="$root/testdata/sources/clingen_cspec" READY="$ready" REQUESTS="$requests" setsid python3 - "$owner_arg" 8>&- <<'PY' >"$fixture_root/server.log" 2>&1 &
import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

CAPTURES = Path(os.environ['CAPTURES'])
READY, REQUESTS = Path(os.environ['READY']), Path(os.environ['REQUESTS'])
MANIFESTS = {
    gene: (CAPTURES / f'{gene.lower()}-manifest.json').read_bytes()
    for gene in ('APC', 'ATM', 'BRCA1', 'MLH1', 'PALB2', 'PTEN', 'TP53', 'BRAF')
}
DOCUMENTS = {
  ('GN020', '1.5.1'): (CAPTURES / 'atm-gn020-1.5.1.json').read_bytes(),
  ('GN003', '3.2.1'): (CAPTURES / 'pten-gn003-3.2.1.json').read_bytes(),
}

class Handler(BaseHTTPRequestHandler):
  def log_message(self, *_): pass
  def send(self, body):
    self.send_response(200); self.send_header('Content-Type', 'application/json'); self.send_header('Content-Length', str(len(body))); self.end_headers(); self.wfile.write(body)
  def do_GET(self):
    path = self.path.split('?', 1)[0]
    with REQUESTS.open('a') as f: f.write(path + '\n')
    parts = [part for part in path.split('/') if part]
    if len(parts) == 6 and parts[:3] == ['cspec', 'Gene', 'id']:
      body = MANIFESTS.get(parts[3])
      if body is not None:
        self.send(body); return
    if len(parts) == 6 and parts[:3] == ['cspec', 'SequenceVariantInterpretation', 'id']:
      body = DOCUMENTS.get((parts[3], parts[5]))
      if body is not None:
        self.send(body); return
    self.send_response(404); self.end_headers()

server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
READY.write_text(f'http://127.0.0.1:{server.server_port}')
server.serve_forever()
PY
pid=$!
cleanup_incomplete_setup() {
  kill -TERM -- "-$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup_incomplete_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
for _ in $(seq 1 50); do test -s "$ready" && break; kill -0 "$pid" 2>/dev/null || { cat "$fixture_root/server.log" >&2; exit 1; }; sleep .1; done
test -s "$ready"
{
  printf 'export BIOMCP_CSPEC_FIXTURE_ORIGIN=%q\n' "$(<"$ready")"
  printf 'export BIOMCP_CACHE_DIR=%q\n' "$fixture_root/cache"
  printf 'export BIOMCP_CSPEC_FIXTURE_REQUESTS=%q\n' "$requests"
} >"$env_file"
bash "$ownership_helper" write "$root" "clingen-cspec" "$fixture_root" "$pid" "BIOMCP_CSPEC_FIXTURE" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
