#!/usr/bin/env bash
# Public workflow driver for spec/entity/clingen-cspec.md. The runner has already
# supplied a local CSpec origin, scratch cache, and request log.
set -euo pipefail
root="$(cd "${1:-../..}" && pwd)"
bin="${BIOMCP_BIN:?run through scripts/run-specs.sh}"
official='https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1'
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

for gene in APC ATM BRCA1 MLH1 PALB2 PTEN TP53 BRAF; do
  "$bin" --json gene cspec "$gene" >"$work/$gene.json"
done
"$bin" --json gene cspec ATM --version "$official" --limit 1 >"$work/selected.json"
capture="$(jq -er '.capture_id' "$work/selected.json")"
requests_before_raw="$(wc -l <"${BIOMCP_CSPEC_FIXTURE_REQUESTS:?fixture requests}")"
"$bin" gene cspec document "$capture" >"$work/raw.json"
"$bin" --json gene cspec ATM --capture-id "$capture" --offset 1 --limit 1 >"$work/page-two.json"
"$bin" --json gene cspec BRCA1 --capture-id "$capture" >"$work/relabel.json" || true
"$bin" --json gene cspec ATM --capture-id 'capture:cspec:sha256:0000000000000000000000000000000000000000000000000000000000000000' >"$work/missing.json" || true

BIN="$bin" CAPTURE="$capture" OUT="$work/mcp.json" uv run --no-sync python - <<'PY'
import json, os, subprocess
proc = subprocess.Popen([os.environ['BIN'], 'mcp'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
def call(value):
    proc.stdin.write(json.dumps(value)+'\n'); proc.stdin.flush(); return json.loads(proc.stdout.readline())
call({'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2025-03-26','capabilities':{},'clientInfo':{'name':'cspec-fixture','version':'1'}}})
proc.stdin.write(json.dumps({'jsonrpc':'2.0','method':'notifications/initialized','params':{}})+'\n'); proc.stdin.flush()
reply=call({'jsonrpc':'2.0','id':2,'method':'tools/call','params':{'name':'gene_cspec','arguments':{'gene':'ATM','capture_id':os.environ['CAPTURE'],'limit':1}}})
open(os.environ['OUT'],'w').write(reply['result']['content'][0]['text'])
proc.terminate(); proc.wait()
PY

uv run --no-sync python - "$work" "${BIOMCP_CSPEC_FIXTURE_REQUESTS:?fixture requests}" "$requests_before_raw" "$root" <<'PY'
import hashlib, json, sys
from pathlib import Path
work, request_log = map(Path, sys.argv[1:3])
requests_before_raw = int(sys.argv[3])
repo_root = Path(sys.argv[4])
recorded_document = (repo_root / 'testdata/sources/clingen_cspec/atm-gn020-1.5.1.json').read_bytes()
manifest_path = '/cspec/Gene/id/ATM/SequenceVariantInterpretation/version'
document_path = '/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1'
def load(name): return json.loads((work/name).read_text())
selected, second, mcp = load('selected.json'), load('page-two.json'), load('mcp.json')
series = {'APC':'GN089','ATM':'GN020','BRCA1':'GN092','MLH1':'GN115','PALB2':'GN077','PTEN':'GN003','TP53':'GN009','BRAF':'GN049'}
def manifest_has(gene, spec): return any(f'/{spec}/version/' in iri for iri in load(f'{gene}.json')['resource_iris'])
raw=(work/'raw.json').read_bytes()
requests=request_log.read_text().splitlines()
report={
 'all_named_gene_series_are_available': all(manifest_has(gene,spec) for gene,spec in series.items()),
 'braf_keeps_gn004_and_gn049': manifest_has('BRAF','GN004') and manifest_has('BRAF','GN049'),
 'atm_uses_literal_full_iri_not_display_version': selected['resource_iri'].endswith('/GN020/version/1.5.1') and selected['display_version']=='1.5',
 'literal_selector_returns_matching_gene_and_specification': selected['gene']=='ATM' and selected['specification_id']=='GN020',
 'criteria_are_deterministic_and_paged': selected['criteria'][0]['label']=='BP6' and second['criteria'][0]['label']=='PM5',
 'supported_reference_objects_preserve_ordered_deduplicated_urls': selected['criteria'][0]['citations']==['https://pubmed.ncbi.nlm.nih.gov/29543229'],
 'receipt_backed_manifest_plan_is_consumed': manifest_path in requests and document_path in requests,
 'receipted_manifest_and_version_page_drive_cli': selected['resource_iri']=='https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1' and selected['source_sha256']==hashlib.sha256(recorded_document).hexdigest(),
 'paged_capture_keeps_provider_criterion_order': selected['criteria'][0]['label']=='BP6' and second['criteria'][0]['label']=='PM5',
 'disease_is_null': selected['disease'] is None,
 'semantic_subset_is_page_independent': selected['semantic_subset_version']=='cspec-semantic-v1' and selected['semantic_subset_sha256']==second['semantic_subset_sha256'],
 'capture_binds_requested_gene_and_selected_iri': selected['capture_binding']['normalized_gene']=='ATM' and selected['capture_binding']['resource_iri']==selected['resource_iri'],
 'cli_capture_page_matches_typed_mcp': selected==mcp,
 'caller_gene_cannot_relabel_capture': load('relabel.json')['error']['code']=='invalid_argument',
 'raw_bytes_match_reported_sha256_and_length': hashlib.sha256(raw).hexdigest()==selected['source_sha256'] and len(raw)==selected['byte_length'],
 'raw_read_does_not_refetch': len(requests) == requests_before_raw,
 'missing_capture_is_capture_unavailable': load('missing.json')['error']['code']=='capture_unavailable',
}
print(json.dumps(report, sort_keys=True))
PY
