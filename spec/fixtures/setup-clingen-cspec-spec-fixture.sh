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
owner_arg="$(bash "$ownership_helper" new-owner "clingen-cspec" "$fixture_root")"
bash "$(dirname "$0")/cleanup-clingen-cspec-spec-fixture.sh" "$root"
ready="$fixture_root/origin"
requests="$fixture_root/requests.jsonl"
: >"$requests"

READY="$ready" REQUESTS="$requests" setsid python3 - "$owner_arg" 8>&- <<'PY' >"$fixture_root/server.log" 2>&1 &
import json, os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

READY, REQUESTS = Path(os.environ['READY']), Path(os.environ['REQUESTS'])
SERIES = {'APC': [('GN089','1.0')], 'ATM': [('GN020','1.5.1')], 'BRCA1': [('GN092','1.0')], 'MLH1': [('GN115','1.0')], 'PALB2': [('GN077','1.0')], 'PTEN': [('GN003','1.0')], 'TP53': [('GN009','1.0')], 'BRAF': [('GN004','1.0'), ('GN049','2.0')]}
def iri(spec, version): return f'https://cspec.clinicalgenome.org/cspec/SequenceVariantInterpretation/id/{spec}/version/{version}'
def envelope(data): return {'status': {'code': 200}, 'metadata': {'fixture': 'cspec-618'}, 'data': data}
def doc(spec, version, gene):
    return envelope({'@id': iri(spec, version), 'entType': 'SequenceVariantInterpretation', 'entId': spec,
      'entContent': {'namespace': spec, 'version': '1.5' if spec == 'GN020' else version, 'states': [{'current': True, 'name': 'current'}]},
      'ldFor': {'Organization': [{'entContent': {'shortTitle': f'{gene} VCEP'}}]},
      'ld': {'CriteriaCode': [
        {'entType':'CriteriaCode','entId':f'{spec}-PS3','entContent':{'sepioID':'SEPIO:0000006','label':'PS3','instructionsToUse':'Use the source criterion as supplied.','references':[{'source':'PubMed','url':'https://pubmed.ncbi.nlm.nih.gov/123456/','id':'123456'}, {'source':'PubMed','url':'https://pubmed.ncbi.nlm.nih.gov/123456/'}]}},
        {'entType':'CriteriaCode','entId':f'{spec}-PM2','entContent':{'sepioID':'SEPIO:0000007','label':'PM2','references':[{'source':'PubMed','url':'https://pubmed.ncbi.nlm.nih.gov/789012/'}]}}
      ]}})
class Handler(BaseHTTPRequestHandler):
  def log_message(self, *_): pass
  def send(self, value):
    body=json.dumps(value,separators=(',',':')).encode(); self.send_response(200); self.send_header('Content-Type','application/json'); self.send_header('Content-Length',str(len(body))); self.end_headers(); self.wfile.write(body)
  def do_GET(self):
    path=self.path.split('?',1)[0]
    with REQUESTS.open('a') as f: f.write(path+'\n')
    parts=[p for p in path.split('/') if p]
    if len(parts)==6 and parts[:3]==['cspec','Gene','id']:
      gene=parts[3]; self.send(envelope([{'@id':iri(spec, version)} for spec,version in SERIES.get(gene, [])])); return
    if len(parts)==6 and parts[:3]==['cspec','SequenceVariantInterpretation','id']:
      spec, version=parts[3],parts[5]
      for gene, rows in SERIES.items():
        if (spec,version) in rows: self.send(doc(spec,version,gene)); return
    self.send_response(404); self.end_headers()
server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
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
