#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="$(realpath -e "${1:-$PWD}")"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-disease-survival-env"
cleanup_script="$script_dir/cleanup-disease-survival-spec-fixture.sh"

mkdir -p "$cache_dir"
cache_dir="$(realpath -e "$cache_dir")"

if [[ -x "$cleanup_script" ]]; then
  bash "$cleanup_script" "$workspace_root"
fi
recover_disease_survival_orphans "$cache_dir"

fixture_root="$(mktemp -d "$cache_dir/spec-disease-survival.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "disease-survival" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_pid_file="$fixture_root/server-pid"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
: >"$request_log"
prepare_fixture_supervisor_owner

start_fixture_supervisor "disease-survival" "$cache_dir" "$fixture_root" "spec-disease-survival." "$server_pid_file" \
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


def source_bytes(path):
    return (SOURCES / path).read_bytes()


SITE_CATALOG = source_bytes("seer/site_catalog_cml.json")
SURVIVAL_PAYLOAD = source_bytes("seer/survival_payload_97_cml.json")
MONARCH_CML_PHENOTYPES_QUERY = {
    "subject": ["MONDO:0011996"],
    "object_category": ["biolink:PhenotypicFeature"],
    "limit": ["80"],
}
PHENOTYPE_IDS_PATH = "/monarch/v3/api/semsim/search/HP:0001250,HP:0001263/Human%20Diseases"
PHENOTYPE_MACRO_PATH = "/monarch/v3/api/semsim/search/HP:0000256/Human%20Diseases"
PHENOTYPE_FIXED_WINDOW = [
    {"subject": {"id": "MONDO:0010450", "name": "intellectual disability, X-linked 89"}, "score": 13.302},
    {"subject": {"id": "MONDO:0007367", "name": "febrile seizures, familial, 1"}, "score": 12.0},
    {"subject": {"id": "MONDO:0000002", "name": "provider-order tie first"}, "score": 11.0},
    {"subject": {"id": "MONDO:0000001", "name": "provider-order tie second"}, "score": 11.0},
    {"subject": {"id": "MONDO:0010450", "name": "later duplicate must lose"}, "score": 99.0},
    {"subject": {"id": "HP:0001250", "name": "non-disease row"}, "score": 10.0},
]
while len(PHENOTYPE_FIXED_WINDOW) < 50:
    index = len(PHENOTYPE_FIXED_WINDOW)
    PHENOTYPE_FIXED_WINDOW.append({
        "subject": {"id": f"MONDO:{index + 2000000:07d}", "name": f"fixture disease {index}"},
        "score": 10.0 - index / 100.0,
    })
OLS_ONTOLOGIES = "hgnc,mesh,mondo,doid,hp,go,chebi,dron,ncit,ordo,wikipathways,so"
OLS_PAYLOADS = {
    "type 2 diabetes mellitus": "ols4/search_type_2_diabetes_mellitus_20260811.json",
    "SCENAR therapy": "ols4/search_scenar_therapy_20260811.json",
    "genes regulated by MEF2 in the heart": "ols4/search_relational_mef2_20260811.json",
}


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    send_bytes(handler, status, body)


def send_bytes(handler, status, body):
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        query = parse_qs(parsed.query)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"GET {self.path}\n")

        if parsed.path == "/healthz":
            send_json(self, 200, {"status": "ok"})
            return
        if parsed.path == "/hpo/search" and query == {"q": ["seizure"]}:
            send_json(self, 200, {"terms": [{"id": "HP:0001250", "name": "Seizure"}]})
            return
        if parsed.path == "/hpo/search" and query == {"q": ["developmental delay"]}:
            send_json(self, 200, {"terms": [{"id": "HP:0001263", "name": "Global developmental delay"}]})
            return
        if parsed.path == "/hpo/search" and query == {"q": ["macrocephaly"]}:
            send_json(self, 200, {"terms": [{"id": "HP:0000256", "name": "Macrocephaly"}]})
            return
        if parsed.path == "/hpo/search" and query == {"q": ["phrase-with-no-hpo-row"]}:
            send_json(self, 200, {"terms": []})
            return
        if parsed.path == "/hpo/terms/HP:0001250":
            send_json(self, 200, {"id": "HP:0001250", "name": "Seizure"})
            return
        if parsed.path == "/hpo/terms/HP:0001263":
            send_json(self, 200, {"id": "HP:0001263", "name": "Global developmental delay"})
            return
        if parsed.path == "/hpo/terms/HP:0000201":
            send_json(self, 200, {"id": "HP:0000201", "name": "Indeterminate fixture phenotype"})
            return
        if parsed.path == "/hpo/terms/HP:0000202":
            send_json(self, 200, {"id": "HP:0000202", "name": "Unavailable fixture phenotype"})
            return
        if parsed.path == "/hpo/terms/HP:0000203":
            send_json(self, 200, {"id": "HP:9999999", "name": "Mismatched fixture phenotype"})
            return
        if parsed.path == "/hpo/terms/HP:0000204":
            send_json(self, 200, {"id": "HP:0000204", "name": "  "})
            return
        if parsed.path == "/ols4/api/search":
            ols_query = query.get("q", [""])[0]
            expected = {
                "q": [ols_query],
                "rows": ["10"],
                "groupField": ["iri"],
                "ontology": [OLS_ONTOLOGIES],
            }
            if ols_query in OLS_PAYLOADS and query == expected:
                send_bytes(self, 200, source_bytes(OLS_PAYLOADS[ols_query]))
                return
        if parsed.path == PHENOTYPE_IDS_PATH and query == {"limit": ["50"]}:
            send_json(self, 200, PHENOTYPE_FIXED_WINDOW)
            return
        if parsed.path == PHENOTYPE_MACRO_PATH and query == {"limit": ["50"]}:
            send_json(self, 200, [
                {"subject":{"id":"MONDO:0019387","name":"isolated microcephaly"},"score":19.0},
                {"subject":{"id":"MONDO:0001234","name":"macrocephaly syndrome"},"score":18.0}
            ])
            return
        if parsed.path in {
            "/monarch/v3/api/semsim/search/HP:0000201/Human%20Diseases",
            "/monarch/v3/api/semsim/search/HP:0000202/Human%20Diseases",
        } and query == {"limit": ["50"]}:
            send_json(self, 200, [{"subject":{"id":"MONDO:0000200","name":"support state fixture disease"},"score":7.0}])
            return
        if parsed.path == "/monarch/v3/api/association":
            required = {
                "category": ["biolink:DiseaseToPhenotypicFeatureAssociation"],
                "predicate": ["biolink:has_phenotype"],
                "object_category": ["biolink:PhenotypicFeature"],
                "direct": ["true"], "limit": ["500"], "offset": ["0"],
            }
            if all(query.get(key) == value for key, value in required.items()):
                subjects = query.get("subject", [])
                objects = query.get("object", [])
                if objects == ["HP:0000201"]:
                    send_json(self, 200, {"items": []})
                    return
                if objects == ["HP:0000202"]:
                    send_json(self, 502, {"error": "fixed association outage"})
                    return
                rows = []
                for subject in subjects:
                    for obj in objects:
                        if subject != "MONDO:0019387":
                            rows.append({
                                "subject": subject, "object": obj,
                                "category": "biolink:DiseaseToPhenotypicFeatureAssociation",
                                "predicate": "biolink:has_phenotype", "negated": False,
                            })
                send_json(self, 200, {"total": len(rows), "items": rows})
                return
        if parsed.path == "/mydisease/query":
            disease_query = query.get("q", [""])[0]
            if "Marfan syndrome" in disease_query:
                send_bytes(self, 200, source_bytes("mydisease/query_marfan_syndrome.json"))
                return
            if (
                "chronic myeloid leukemia" in disease_query
                or "chronic myelogenous leukemia" in disease_query
            ):
                send_bytes(
                    self,
                    200,
                    source_bytes("mydisease/query_chronic_myeloid_leukemia.json"),
                )
                return
        if parsed.path == "/mydisease/disease/MONDO:0011996":
            send_bytes(self, 200, source_bytes("mydisease/get_mondo_0011996.json"))
            return
        if parsed.path == "/mydisease/disease/MONDO:0007947":
            send_bytes(self, 200, source_bytes("mydisease/get_mondo_0007947.json"))
            return
        if parsed.path == "/monarch/v3/api/association" and query == MONARCH_CML_PHENOTYPES_QUERY:
            send_bytes(self, 200, source_bytes("monarch/association_mondo_0011996_phenotypes.json"))
            return
        if parsed.path == "/seer/get_var_formats.php":
            send_bytes(self, 200, SITE_CATALOG)
            return
        if parsed.path == "/seer/render_region_5.php":
            send_bytes(self, 200, SURVIVAL_PAYLOAD)
            return

        send_json(self, 404, {"error": "fixture path not found"})

    def do_POST(self):
        parsed = urlparse(self.path)
        length = int(self.headers.get("Content-Length", "0"))
        body = json.loads(self.rfile.read(length) or b"{}")
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"POST {self.path} {json.dumps(body, separators=(',', ':'))}\n")
        search_text = body.get("criteria", {}).get("advanced_text_search", {}).get("search_text")
        if parsed.path == "/nih/projects/search" and search_text == '"Marfan syndrome"':
            send_bytes(self, 200, source_bytes("nih_reporter/funding_marfan_syndrome.json"))
            return
        send_json(self, 404, {"error": "fixture path not found"})

    def log_message(self, format, *args):
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
  if [[ -s "$server_pid_file" ]]; then
    server_pid="$(cat "$server_pid_file")"
  fi
  if [[ -s "$ready_file" && "$server_pid" =~ ^[1-9][0-9]*$ ]]; then
    break
  fi
  if ! kill -0 "$supervisor_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

test -s "$ready_file"
[[ "$server_pid" =~ ^[1-9][0-9]*$ ]]
base_url="$(cat "$ready_file")"

for _ in $(seq 1 50); do
  if python3 - "$base_url/healthz" <<'PY' >/dev/null 2>&1
from urllib.request import urlopen
import sys

with urlopen(sys.argv[1], timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY
  then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

python3 - "$base_url/healthz" <<'PY' >/dev/null
from urllib.request import urlopen
import sys

with urlopen(sys.argv[1], timeout=1) as response:
    if response.status != 200:
        raise SystemExit(1)
PY

{
  printf 'export BIOMCP_MYDISEASE_BASE=%q\n' "$base_url/mydisease"
  printf 'export BIOMCP_MONARCH_BASE=%q\n' "$base_url/monarch"
  printf 'export BIOMCP_HPO_BASE=%q\n' "$base_url/hpo"
  printf 'export BIOMCP_OLS4_BASE=%q\n' "$base_url/ols4"
  printf 'export BIOMCP_MEDLINEPLUS_BASE=%q\n' "$base_url/unused-medlineplus"
  printf 'export UMLS_API_KEY=%q\n' ''
  printf 'export BIOMCP_NIH_REPORTER_BASE=%q\n' "$base_url/nih"
  printf 'export BIOMCP_SEER_BASE=%q\n' "$base_url/seer"
  printf 'export BIOMCP_DGIDB_BASE=%q\n' "$base_url/unused-dgidb"
  printf 'export BIOMCP_OPENTARGETS_BASE=%q\n' "$base_url/unused-opentargets"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_DISEASE_SURVIVAL_READY_FILE=%q\n' "$ready_file"
  printf 'export BIOMCP_DISEASE_SURVIVAL_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "disease-survival" "$fixture_root" "$server_pid" "BIOMCP_DISEASE_SURVIVAL" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
