#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="$(realpath -e "${1:-$PWD}")"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-provider-contract-env"
cleanup_script="$script_dir/cleanup-provider-contract-spec-fixture.sh"
mkdir -p "$cache_dir"
cache_dir="$(realpath -e "$cache_dir")"
bash "$cleanup_script" "$workspace_root"
recover_provider_contract_orphans "$cache_dir"

fixture_root="$(mktemp -d "$cache_dir/spec-provider-contract.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "provider-contract" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_pid_file="$fixture_root/server-pid"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
ema_dir="$fixture_root/ema-human"
who_dir="$fixture_root/who-pq"
who_ivd_dir="$fixture_root/who-ivd"
gtr_dir="$fixture_root/gtr"
: >"$request_log"
cp -R "$script_dir/ema-human" "$ema_dir"
cp -R "$script_dir/who-pq" "$who_dir"
cp -R "$script_dir/gtr" "$gtr_dir"
cp -R "$script_dir/who-ivd" "$who_ivd_dir"
find "$ema_dir" "$who_dir" "$who_ivd_dir" "$gtr_dir" -type f -exec touch {} +
prepare_fixture_supervisor_owner

start_fixture_supervisor "provider-contract" "$cache_dir" "$fixture_root" "spec-provider-contract." "$server_pid_file" \
  python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import json
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
SOURCES = ROOT / "testdata/sources"


def fixture(path):
    return (SOURCES / path).read_bytes()


MYCHEM = {
    "Keytruda": fixture("mychem/query_keytruda_get_20260811.json"),
    "pembrolizumab": fixture("mychem/query_pembrolizumab_get_20260811.json"),
    "trastuzumab": fixture("mychem/query_trastuzumab_search_20260811.json"),
    'drugcentral.drug_use.indication.concept_name:"Marfan syndrome"': fixture(
        "mychem/query_marfan_indication_20260811.json"
    ),
    "imatinib": fixture("mychem/query_imatinib_get_20260811.json"),
    "warfarin": fixture("mychem/query_warfarin_get_20260811.json"),
    "daraxonrasib": fixture("mychem/query_daraxonrasib_get_20260811.json"),
    "dabigatran": fixture("mychem/query_dabigatran_get_20260811.json"),
}
MYGENE = {
    "(symbol:BRAF OR alias:BRAF)": fixture("mygene/search_braf_20260811.json"),
    "(symbol:H3\\-3A OR alias:H3\\-3A)": json.dumps({
        "total": 1,
        "hits": [{"_id": "3020", "symbol": "H3-3A", "alias": ["H3F3A"]}],
    }).encode("utf-8"),
    "(symbol:NOTAREALGENE1091 OR alias:NOTAREALGENE1091)": b'{"total":0,"hits":[]}',
    'symbol:"BRAF"': fixture("mygene/get_braf_20260811.json"),
    'symbol:"PD\\-L1"': fixture("mygene/get_pdl1_empty_20260811.json"),
    "(symbol:PD\\-L1 OR alias:PD\\-L1)": fixture("mygene/search_pdl1_20260811.json"),
    'symbol:"CD274"': fixture("mygene/get_cd274_20260811.json"),
    'symbol:"BRCA1"': fixture("mygene/get_brca1_20260811.json"),
    'symbol:"EGFR"': fixture("mygene/get_egfr_20260811.json"),
    'symbol:"ERBB2"': fixture("mygene/get_erbb2_20260811.json"),
    'symbol:"TP53"': json.dumps({
        "total": 1,
        "hits": [{
            "symbol": "TP53",
            "name": "tumor protein p53",
            "entrezgene": 7157,
            "type_of_gene": "protein-coding",
            "ensembl": {"gene": "ENSG00000141510"},
            "uniprot": {"Swiss-Prot": "P04637"},
        }],
    }).encode("utf-8"),
}
CLINGEN_LOOKUP_TP53 = fixture("clingen/lookup_tp53.json")
CLINGEN_VALIDITY_TP53 = fixture("clingen/validity_tp53.csv")
CLINGEN_DOSAGE_TP53 = fixture("clingen/dosage_tp53.csv")
OPENFDA_LABEL = fixture("openfda/label_keytruda_20260811.json")
OPENFDA_DRUGSFDA = fixture("openfda/drugsfda_imatinib_20260811.json")
OPENFDA_DEVICE_510K = fixture("openfda/device_510k_brca1_20260811.json")
OPENFDA_DEVICE_PMA = fixture("openfda/device_pma_brca1_20260811.json")
OPENFDA_FAERS_EVENT = fixture("openfda/faers_event.json")
OPENFDA_FAERS_COUNT = fixture(
    "openfda/faers_count_pembrolizumab_reaction_20260811.json"
)
CHEMBL_MECHANISMS = fixture("chembl/mechanisms_pembrolizumab_20260811.json")
OPENTARGETS_DRUG = fixture("opentargets/drug_pembrolizumab_20260811.json")
QUICKGO_ANNOTATIONS = fixture("quickgo/annotations_braf_20260811.json")
QUICKGO_TERMS = fixture("quickgo/terms_braf_20260811.json")
STRING_NETWORK = fixture("string/network_braf_20260811.json")
HPA_BRAF = fixture("hpa/braf_20260811.xml")
DGIDB_EGFR = fixture("dgidb/gene_egfr_20260811.json")
NIH_ERBB2 = fixture("nih_reporter/funding_erbb2_20260811.json")
NIH_MARFAN = fixture("nih_reporter/funding_marfan_syndrome.json")
OPENTARGETS = {
    ("ENSG00000157764", False): fixture("opentargets/clinical_braf_20260811.json"),
    ("ENSG00000146648", False): fixture("opentargets/clinical_egfr_20260811.json"),
    ("ENSG00000146648", True): fixture("opentargets/druggability_egfr_20260811.json"),
    ("ENSG00000141736", False): fixture("opentargets/clinical_erbb2_20260811.json"),
    ("ENSG00000012048", False): fixture("opentargets/clinical_brca1_20260811.json"),
    ("ENSG00000141510", False): b'{"data":{"target":{"associatedDiseases":{"rows":[]},"drugAndClinicalCandidates":{"rows":[]}}}}',
}
KEGG_SEARCH = fixture("kegg/search_mapk_20260811.txt")
KEGG_DETAIL = fixture("kegg/get_hsa05200_20260811.txt")
REACTOME_SEARCH = {
    "3": fixture("reactome/search_mapk_limit3_20260811.json"),
    "5": fixture("reactome/search_mapk_limit5_20260811.json"),
}
REACTOME_DETAIL = fixture("reactome/get_r_hsa_5673001_20260811.json")
REACTOME_PARTICIPANTS = fixture("reactome/participants_r_hsa_5673001_20260811.json")
REACTOME_EVENTS = fixture("reactome/events_r_hsa_5673001_20260811.json")
WIKIPATHWAYS_SEARCH = b'{"result":[]}'
NCI_MELANOMA = fixture("nci_cts/search_melanoma_20260811.json")


def send(handler, status, body, content_type="application/json"):
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"GET {self.path}\n")

        if parsed.path == "/healthz":
            send(self, 200, b'{"status":"ok"}')
            return
        if parsed.path == "/mychem/v1/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            if query == "fixture-provider-failure":
                send(self, 503, b'{"error":"synthetic provider failure"}')
                return
            body = MYCHEM.get(query)
            if body is not None:
                send(self, 200, body)
                return
        if parsed.path == "/mygene/v3/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            body = MYGENE.get(query)
            if body is not None:
                send(self, 200, body)
                return
        if parsed.path.endswith("/api/genes/look/TP53") and parsed.path.startswith("/clingen/"):
            send(self, 200, CLINGEN_LOOKUP_TP53)
            return
        if parsed.path.endswith("/kb/gene-validity/download") and parsed.path.startswith("/clingen/"):
            if parsed.path.startswith("/clingen/validity-fail/"):
                send(self, 400, b"synthetic private validity failure")
            else:
                send(self, 200, CLINGEN_VALIDITY_TP53, "text/csv")
            return
        if parsed.path.endswith("/kb/gene-dosage/download") and parsed.path.startswith("/clingen/"):
            if parsed.path.startswith("/clingen/dosage-timeout/"):
                import time
                time.sleep(0.2)
            send(self, 200, CLINGEN_DOSAGE_TP53, "text/csv")
            return
        if parsed.path == "/openfda/drug/label.json":
            search = parse_qs(parsed.query).get("search", [""])[0].lower()
            if "keytruda" in search or "pembrolizumab" in search:
                send(self, 200, OPENFDA_LABEL)
                return
        if parsed.path == "/openfda/drug/drugsfda.json":
            send(self, 200, OPENFDA_DRUGSFDA)
            return
        if parsed.path == "/openfda/drug/event.json":
            query = parse_qs(parsed.query)
            if query.get("count") == ["patient.reaction.reactionmeddrapt.exact"]:
                send(self, 200, OPENFDA_FAERS_COUNT)
            else:
                send(self, 200, OPENFDA_FAERS_EVENT)
            return
        if parsed.path == "/openfda/device/510k.json":
            query = parse_qs(parsed.query)
            if query.get("limit") == ["25"] and "BRCA1 Hereditary Cancer Panel" in query.get("search", [""])[0]:
                send(self, 404, OPENFDA_DEVICE_510K)
                return
        if parsed.path == "/openfda/device/pma.json":
            query = parse_qs(parsed.query)
            if query.get("limit") == ["25"] and "BRCA1 Hereditary Cancer Panel" in query.get("search", [""])[0]:
                send(self, 404, OPENFDA_DEVICE_PMA)
                return
        if parsed.path == "/chembl/mechanism.json":
            query = parse_qs(parsed.query)
            if query == {"molecule_chembl_id": ["CHEMBL3137343"], "limit": ["15"]}:
                send(self, 200, CHEMBL_MECHANISMS)
                return
        if parsed.path == "/quickgo/QuickGO/services/annotation/search":
            if parse_qs(parsed.query) == {"geneProductId": ["P15056"], "limit": ["20"]}:
                send(self, 200, QUICKGO_ANNOTATIONS)
                return
        if parsed.path == "/quickgo/QuickGO/services/ontology/go/terms/GO:0004672,GO:0004674,GO:0004708,GO:0004709,GO:0005509,GO:0005515,GO:0031267":
            send(self, 200, QUICKGO_TERMS)
            return
        if parsed.path == "/string/api/json/network":
            if parse_qs(parsed.query) == {"identifiers": ["BRAF"], "species": ["9606"], "limit": ["15"]}:
                send(self, 200, STRING_NETWORK)
                return
        if parsed.path == "/hpa/ENSG00000157764.xml":
            send(self, 200, HPA_BRAF, "application/xml")
            return
        if parsed.path == "/kegg/find/pathway/MAPK%20signaling%20pathway":
            send(self, 200, KEGG_SEARCH, "text/plain")
            return
        if parsed.path == "/kegg/get/hsa05200":
            send(self, 200, KEGG_DETAIL, "text/plain")
            return
        if parsed.path == "/reactome/ContentService/search/query":
            query = parse_qs(parsed.query)
            if (
                query.get("query") == ["MAPK signaling pathway"]
                and query.get("species") == ["Homo sapiens"]
                and query.get("pageSize", [""])[0] in REACTOME_SEARCH
            ):
                send(self, 200, REACTOME_SEARCH[query["pageSize"][0]])
                return
        if parsed.path == "/reactome/ContentService/data/query/R-HSA-5673001":
            send(self, 200, REACTOME_DETAIL)
            return
        if parsed.path == "/reactome/ContentService/data/participants/R-HSA-5673001":
            send(self, 200, REACTOME_PARTICIPANTS)
            return
        if parsed.path == "/reactome/ContentService/data/pathway/R-HSA-5673001/containedEvents":
            send(self, 200, REACTOME_EVENTS)
            return
        if parsed.path == "/wikipathways/findPathwaysByText.json":
            send(self, 200, WIKIPATHWAYS_SEARCH)
            return
        if parsed.path == "/nci/api/v2/trials":
            query = parse_qs(parsed.query)
            if query == {"keyword": ["melanoma"], "size": ["1"], "from": ["0"]}:
                send(self, 200, NCI_MELANOMA)
                return

        send(self, 404, b'{"error":"fixture route not found"}')

    def do_POST(self):
        parsed = urlparse(self.path)
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"POST {self.path} {body.decode('utf-8')}\n")
        if parsed.path == "/opentargets/api/v4/graphql":
            request = json.loads(body)
            if request.get("variables") == {"chemblId": "CHEMBL3137343"}:
                send(self, 200, OPENTARGETS_DRUG)
                return
            variables = request.get("variables", {})
            ensembl_id = variables.get("ensemblId")
            is_druggability = "tractability" in request.get("query", "")
            response = OPENTARGETS.get((ensembl_id, is_druggability))
            if response is not None:
                send(self, 200, response)
                return
        if parsed.path == "/dgidb/api/graphql":
            request = json.loads(body)
            if request.get("variables") == {"gene": "EGFR", "first": 1}:
                send(self, 200, DGIDB_EGFR)
                return
        if parsed.path == "/nih/v2/projects/search":
            request = json.loads(body)
            search = request.get("criteria", {}).get("advanced_text_search", {})
            if search.get("search_text") == '"ERBB2"':
                send(self, 200, NIH_ERBB2)
                return
            if search.get("search_text") == '"Marfan syndrome"':
                send(self, 200, NIH_MARFAN)
                return
        send(self, 404, b'{"error":"fixture route not found"}')

    def log_message(self, _format, *_args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
supervisor_pid=$!
server_pid=""
cleanup_incomplete_setup() {
  if [[ -s "$server_pid_file" ]]; then
    server_pid="$(cat "$server_pid_file")"
    [[ "$server_pid" =~ ^[1-9][0-9]*$ ]] && kill -TERM -- "-$server_pid" 2>/dev/null || true
    wait "$supervisor_pid" 2>/dev/null || true
  else
    kill -TERM "$supervisor_pid" 2>/dev/null || true
    wait "$supervisor_pid" 2>/dev/null || true
    rm -rf "$fixture_root"
  fi
}
trap cleanup_incomplete_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for _ in $(seq 1 50); do
  [[ -s "$server_pid_file" ]] && server_pid="$(cat "$server_pid_file")"
  [[ -s "$ready_file" && "$server_pid" =~ ^[1-9][0-9]*$ ]] && break
  kill -0 "$supervisor_pid" 2>/dev/null || { cat "$server_log" >&2; exit 1; }
  sleep 0.1
done
test -s "$ready_file"
[[ "$server_pid" =~ ^[1-9][0-9]*$ ]]
base_url="$(cat "$ready_file")"
for _ in $(seq 1 50); do
  if curl --fail --silent "$base_url/healthz" >/dev/null; then break; fi
  kill -0 "$server_pid" 2>/dev/null || { cat "$server_log" >&2; exit 1; }
  sleep 0.1
done
curl --fail --silent "$base_url/healthz" >/dev/null

{
  printf 'export BIOMCP_MYCHEM_BASE=%q\n' "$base_url/mychem/v1"
  printf 'export BIOMCP_MYGENE_BASE=%q\n' "$base_url/mygene/v3"
  printf 'export BIOMCP_OPENFDA_BASE=%q\n' "$base_url/openfda"
  printf 'export BIOMCP_CHEMBL_BASE=%q\n' "$base_url/chembl"
  printf 'export BIOMCP_OPENTARGETS_BASE=%q\n' "$base_url/opentargets/api/v4"
  printf 'export BIOMCP_QUICKGO_BASE=%q\n' "$base_url/quickgo/QuickGO/services"
  printf 'export BIOMCP_STRING_BASE=%q\n' "$base_url/string/api"
  printf 'export BIOMCP_HPA_BASE=%q\n' "$base_url/hpa"
  printf 'export BIOMCP_DGIDB_BASE=%q\n' "$base_url/dgidb/api"
  printf 'export BIOMCP_NIH_REPORTER_BASE=%q\n' "$base_url/nih/v2"
  printf 'export BIOMCP_KEGG_BASE=%q\n' "$base_url/kegg"
  printf 'export BIOMCP_REACTOME_BASE=%q\n' "$base_url/reactome/ContentService"
  printf 'export BIOMCP_WIKIPATHWAYS_BASE=%q\n' "$base_url/wikipathways"
  printf 'export BIOMCP_NCI_CTS_BASE=%q\n' "$base_url/nci/api/v2"
  printf 'export NCI_API_KEY=%q\n' 'fixture-nci-key'
  printf 'export BIOMCP_EMA_DIR=%q\n' "$ema_dir"
  printf 'export BIOMCP_WHO_DIR=%q\n' "$who_dir"
  printf 'export BIOMCP_WHO_IVD_DIR=%q\n' "$who_ivd_dir"
  printf 'export BIOMCP_GTR_DIR=%q\n' "$gtr_dir"
  printf 'export BIOMCP_PROVIDER_CONTRACT_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_TEST_UNPACED_ORIGIN=%q\n' "$base_url"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_PROVIDER_CONTRACT_READY_FILE=%q\n' "$ready_file"
  printf 'export BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "provider-contract" "$fixture_root" "$server_pid" "BIOMCP_PROVIDER_CONTRACT" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
