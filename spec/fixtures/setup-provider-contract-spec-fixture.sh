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
gencc_tmp_root="${TMPDIR:-/tmp}"
if [[ "${BIOMCP_OFFLINE_NETWORK:-0}" == 1 ]]; then
  gencc_tmp_root=/tmp
fi
gencc_parent="$(mktemp -d "$gencc_tmp_root/biomcp-gencc-provider-contract.XXXXXX")"
gencc_dir="$gencc_parent/gencc"
cleanup_gencc_parent() {
  if [[ "$gencc_parent" == "$gencc_tmp_root/biomcp-gencc-provider-contract."* && -d "$gencc_parent" && ! -L "$gencc_parent" ]]; then
    rm -rf -- "$gencc_parent"
  fi
}
trap cleanup_gencc_parent EXIT
if [[ "${BIOMCP_TEST_PROVIDER_FIXTURE_FAIL_AFTER_GENCC:-}" == 1 ]]; then
  exit 86
fi
: >"$request_log"
cp -R "$script_dir/ema-human" "$ema_dir"
cp -R "$script_dir/who-pq" "$who_dir"
cp -R "$script_dir/gtr" "$gtr_dir"
cp -R "$script_dir/who-ivd" "$who_ivd_dir"
find "$ema_dir" "$who_dir" "$who_ivd_dir" "$gtr_dir" -type f -exec touch {} +
python3 - "$gtr_dir/test_version.gz" <<'PY'
import gzip
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
body = gzip.decompress(path.read_bytes()).decode("utf-8")
body += (
    "GTR000596648.2\t1\tBachmann-Bupp diagnostic panel\tExample Diagnostics\t"
    "molecular\tExample Lab\tExample Institute\t00D5966482\tNY\tUSA\tCurrent\tPublic\t"
    "Molecular genetics\tSequence analysis\tODC1\tMONDO:0033642\n"
    "GTR000596649.1\t1\tZulu Bachmann-Bupp assay\tExample Diagnostics\t"
    "molecular\tAnother Lab\tExample Institute\t00D5966491\tNY\tUSA\tCurrent\tPublic\t"
    "Molecular genetics\tSequence analysis\tODC1\tMONDO:0033642\n"
)
path.write_bytes(gzip.compress(body.encode("utf-8")))
PY
prepare_fixture_supervisor_owner

start_fixture_supervisor "provider-contract" "$cache_dir" "$fixture_root" "spec-provider-contract." "$server_pid_file" \
  python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, parse_qsl, urlparse
import json
import re
import sys

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
SOURCES = ROOT / "testdata/sources"
WORKER_NAMESPACE = re.compile(
    r"^/__biomcp_provider_worker/(request-log\.[A-Za-z0-9]{6,})(/.*)$"
)


def resolve_request(request_target):
    parsed = urlparse(request_target)
    match = WORKER_NAMESPACE.fullmatch(parsed.path)
    if not match:
        return parsed, request_target, REQUEST_LOG
    request_log = REQUEST_LOG.parent / match.group(1)
    if not request_log.is_file():
        return None
    canonical = parsed._replace(path=match.group(2))
    return canonical, canonical.geturl(), request_log


def fixture(path):
    return (SOURCES / path).read_bytes()


MYCHEM = {
    "Keytruda": fixture("mychem/query_keytruda_get_20260811.json"),
    "pembrolizumab": fixture("mychem/query_pembrolizumab_get_20260811.json"),
    "trastuzumab": fixture("mychem/query_trastuzumab_search_20260811.json"),
    "eflornithine": fixture("mychem/query_eflornithine_search_20260905.json"),
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
    "(symbol:ODC1 OR alias:ODC1)": json.dumps({
        "total": 4,
        "hits": [
            {"symbol": "SLC25A21", "name": "solute carrier family 25 member 21", "entrezgene": 23530},
            {"symbol": "ODC1", "name": "ornithine decarboxylase 1", "entrezgene": 4953},
            {"symbol": "ODC1", "name": "duplicate ODC1", "entrezgene": 4953},
            {"symbol": "OAZ1", "name": "ornithine decarboxylase antizyme 1", "entrezgene": 4946},
        ],
    }).encode("utf-8"),
    "(symbol:ALIASONLY OR alias:ALIASONLY)": json.dumps({
        "total": 1,
        "hits": [{"symbol": "CANONICAL_ALIAS", "alias": ["ALIASONLY"],
                  "name": "canonical alias target", "entrezgene": 9050}],
    }).encode("utf-8"),
    "(symbol:NOEXACT OR alias:NOEXACT)": json.dumps({
        "total": 6,
        "hits": [
            {"name": "missing symbol and id"},
            {"symbol": "NOEXACT_A", "name": "missing id"},
            {"symbol": "", "name": "blank symbol", "entrezgene": 9303},
            {"symbol": "NOEXACT_B", "name": "later row", "entrezgene": 9304},
            {"symbol": "NOEXACT_B", "name": "duplicate later row", "entrezgene": 9304},
            {"symbol": "", "name": "second blank symbol"},
        ],
    }).encode("utf-8"),
    "odc1": json.dumps({
        "total": 4,
        "hits": [
            {"symbol": "SLC25A21", "name": "solute carrier family 25 member 21", "entrezgene": 23530},
            {"symbol": "ODC1", "name": "ornithine decarboxylase 1", "entrezgene": 4953},
            {"symbol": "ODC1", "name": "duplicate ODC1", "entrezgene": 4953},
            {"symbol": "OAZ1", "name": "ornithine decarboxylase antizyme 1", "entrezgene": 4946},
        ],
    }).encode("utf-8"),
    "OdC1": json.dumps({
        "total": 4,
        "hits": [
            {"symbol": "SLC25A21", "name": "solute carrier family 25 member 21", "entrezgene": 23530},
            {"symbol": "ODC1", "name": "ornithine decarboxylase 1", "entrezgene": 4953},
            {"symbol": "ODC1", "name": "duplicate ODC1", "entrezgene": 4953},
            {"symbol": "OAZ1", "name": "ornithine decarboxylase antizyme 1", "entrezgene": 4946},
        ],
    }).encode("utf-8"),
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
    'symbol:"FLT3"': fixture("mygene/get_flt3_20260918.json"),
    'symbol:"EGFR"': fixture("mygene/get_egfr_20260811.json"),
    'symbol:"ERBB2"': fixture("mygene/get_erbb2_20260811.json"),
    'symbol:"ODC1"': json.dumps({
        "total": 1,
        "hits": [{
            "symbol": "ODC1",
            "name": "ornithine decarboxylase 1",
            "entrezgene": 4953,
            "type_of_gene": "protein-coding",
            "ensembl": {"gene": "ENSG00000115758"},
            "HGNC": 8109,
        }],
    }).encode("utf-8"),
    'symbol:"GENCCIDENTITY"': json.dumps({
        "total": 1,
        "hits": [{
            "symbol": "GENCCIDENTITY",
            "name": "synthetic GenCC identity conflict",
            "entrezgene": 981158,
            "ensembl": {"gene": "ENSG00000115758"},
            "HGNC": 8109,
        }],
    }).encode("utf-8"),
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


def synthetic_gene_response(total, offset=0, size=50):
    """Deterministic rows for bounded acquisition and overflow contracts."""
    hits = []
    for index in range(offset, min(offset + size, total)):
        # Keep a duplicate in the overflow page: overflow must preserve the
        # provider page exactly, while complete sets deduplicate after rank.
        symbol = "OVERFLOW_DUP" if index < 2 else f"ALIAS{index}"
        if total == 50 and index == 49:
            symbol = "TOTAL50"
        if total == 51 and index == 50:
            symbol = "TOTAL51"
        entrezgene = 9000 if total == 51 and index < 2 else 9000 + index
        hits.append({"symbol": symbol, "name": f"synthetic {symbol}", "entrezgene": entrezgene})
    return json.dumps({"total": total, "hits": hits}).encode("utf-8")


FILTER_HITS = json.dumps({
    "total": 5,
    "hits": [
        {"symbol": "FILTERCASE", "name": "matching row", "entrezgene": 9101,
         "type_of_gene": "protein-coding", "genomic_pos": {"chr": "chr7", "start": 100, "end": 200}},
        {"symbol": "FILTER_WRONG_TYPE", "name": "wrong type", "entrezgene": 9102,
         "type_of_gene": "ncRNA", "genomic_pos": {"chr": "7", "start": 100, "end": 200}},
        {"symbol": "FILTER_WRONG_CHR", "name": "wrong chromosome", "entrezgene": 9103,
         "type_of_gene": "protein-coding", "genomic_pos": {"chr": "8", "start": 100, "end": 200}},
        {"symbol": "FILTER_OUTSIDE", "name": "outside region", "entrezgene": 9104,
         "type_of_gene": "protein-coding", "genomic_pos": {"chr": "7", "start": 1000, "end": 1100}},
        {"symbol": "FILTER_ALIAS", "name": "alias row", "entrezgene": 9105,
         "type_of_gene": "protein-coding", "genomic_pos": {"chr": "7", "start": 150, "end": 250}},
    ],
}).encode("utf-8")
HOSTILE_SYMBOL = 'bad " quote \\ slash $HOME `tick`; &amp'
HOSTILE = json.dumps({"total": 1, "hits": [{"symbol": HOSTILE_SYMBOL, "name": "hostile provider symbol", "entrezgene": 9201}]}).encode("utf-8")
FILTER_PATHWAY_QUERY = '(symbol:FILTERCASE OR alias:FILTERCASE) AND (pathway.kegg.id:"R\\-HSA\\-5673001" OR pathway.reactome.id:"R\\-HSA\\-5673001" OR pathway.kegg.name:*R\\-HSA\\-5673001*)'
FILTER_GO_QUERY = '(symbol:FILTERCASE OR alias:FILTERCASE) AND (go.BP.id:"GO\\:0004672" OR go.CC.id:"GO\\:0004672" OR go.MF.id:"GO\\:0004672")'
LUCENE_QUERY = '+-=&&||><!(){}[]^"~*?:/\\'
LUCENE_ESCAPED = ''.join(f"\\{char}" if char in r'\\+-!(){}[]^"~*?:/&|' else char for char in LUCENE_QUERY)
CLINGEN_LOOKUP_TP53 = fixture("clingen/lookup_tp53.json")
CLINGEN_VALIDITY_TP53 = fixture("clingen/validity_tp53.csv")
CLINGEN_DOSAGE_TP53 = fixture("clingen/dosage_tp53.csv")
GENCC_ODC1 = fixture("gencc/submissions-new-odc1.csv")
GENCC_ETAG = '"6ebdbf28b305e99e349ed827a219214b"'
GENCC_LAST_MODIFIED = "Sun, 06 Sep 2026 06:00:29 GMT"
OPENFDA_LABEL = fixture("openfda/label_keytruda_20260811.json")
OPENFDA_DRUGSFDA = fixture("openfda/drugsfda_imatinib_20260811.json")
OPENFDA_DEVICE_510K = fixture("openfda/device_510k_brca1_20260811.json")
OPENFDA_DEVICE_PMA = fixture("openfda/device_pma_brca1_20260811.json")
OPENFDA_FAERS_EVENT = fixture("openfda/faers_event.json")
OPENFDA_FAERS_COUNT = fixture(
    "openfda/faers_count_pembrolizumab_reaction_20260811.json"
)
FDA_ORPHAN = fixture("fda_orphan/provider-shaped.html")
CHEMBL_MECHANISMS = fixture("chembl/mechanisms_pembrolizumab_20260811.json")
CHEMBL_STATUS = fixture("chembl/status_20260918.json")
CHEMBL_CELL_LINE_EMPTY = fixture("chembl/cell_line_none_20260918.json")
CHEMBL_CELL_LINES = {
    "CVCL_2119": fixture("chembl/cell_line_cvcl_2119_20260918.json"),
    "CVCL_0004": fixture("chembl/cell_line_cvcl_0004_20260918.json"),
}
CHEMBL_ASSAY_COUNTS = {
    "CHEMBL3706573": fixture("chembl/assay_count_chembl3706573_20260918.json"),
}
OPENTARGETS_DRUG = fixture("opentargets/drug_pembrolizumab_20260811.json")
QUICKGO_ANNOTATIONS = fixture("quickgo/annotations_braf_20260811.json")
QUICKGO_TERMS = fixture("quickgo/terms_braf_20260811.json")
STRING_NETWORK = fixture("string/network_braf_20260811.json")
HPA_BRAF = fixture("hpa/braf_20260811.xml")
HPA_CELL_RNA = {
    ("ENSG00000122025", "g,eg,cell_RNA_leukemia"): fixture(
        "hpa/cell_rna_leukemia_flt3_20260918.json"
    ),
}
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
CELLOSAURUS_RELEASE = fixture("cellosaurus/release_info_20260917.json")
CELLOSAURUS_EMPTY = b'{"Cellosaurus":{"cell-line-list":[]}}'
CELLOSAURUS_RECORDS = {
    "CVCL_2119": fixture("cellosaurus/get_cvcl_2119_20260917.json"),
    "CVCL_0064": fixture("cellosaurus/get_cvcl_0064_20260917.json"),
    "CVCL_1844": fixture("cellosaurus/get_cvcl_1844_var_20260917.json"),
    "CVCL_0007": fixture("cellosaurus/get_cvcl_0007_20260917.json"),
    "CVCL_0005": fixture("cellosaurus/get_cvcl_0005_20260917.json"),
}
# Ticket 1205: PharmacoDB answers every lookup and both experiment reads from
# recorded bodies. The counts projection and the row projection are the same
# `experiments` query with different fields, so the router tells them apart by
# the metric fields.
PHARMACODB_CELL_LINE_BY_UID = {
    # PharmacoDB's own UID for MOLM-13, which is the value the Cellosaurus
    # PharmacoDB cross-reference carries.
    "MOLM13_950_2019": fixture("pharmacodb/cell_line_uid_molm13_20260918.json"),
}
PHARMACODB_CELL_LINE_BY_NAME = {
    "HL-60(TB)": fixture("pharmacodb/cell_line_name_hl60tb_20260918.json"),
    "HL-60": fixture("pharmacodb/cell_line_name_hl60_20260918.json"),
}
PHARMACODB_CELL_LINE_MISSING = fixture("pharmacodb/cell_line_name_missing_20260918.json")
PHARMACODB_COMPOUNDS = {
    "venetoclax": fixture("pharmacodb/compound_venetoclax_20260918.json"),
}
PHARMACODB_COMPOUND_MISSING = fixture("pharmacodb/compound_missing_20260918.json")
PHARMACODB_COUNTS = {
    (1248, None): fixture("pharmacodb/experiment_counts_cell_line_1248_20260918.json"),
    (None, 53572): fixture("pharmacodb/experiment_counts_compound_53572_20260918.json"),
}
PHARMACODB_EXPERIMENTS = {
    (1248, None): fixture("pharmacodb/experiments_cell_line_1248_20260918.json"),
    (None, 53572): fixture("pharmacodb/experiments_compound_53572_20260918.json"),
    (1248, 53572): fixture("pharmacodb/experiments_pair_53572_1248_20260918.json"),
}

# Ticket 1213: the five identifier batches the 93-line leukemia group needs.
# The key is the first name in the batch, which the query quotes first.
CELLOSAURUS_ID_BATCHES = {
    "697": fixture("cellosaurus/search_id_leukemia_batch1_20260918.json"),
    "JK-1": fixture("cellosaurus/search_id_leukemia_batch2_20260918.json"),
    "ME-1 [Human leukemia]": fixture("cellosaurus/search_id_leukemia_batch3_20260918.json"),
    "NALM-19": fixture("cellosaurus/search_id_leukemia_batch4_20260918.json"),
    "SEM": fixture("cellosaurus/search_id_leukemia_batch5_20260918.json"),
}
CELLOSAURUS_SEARCHES = {
    'idsy:"MOLM13"': fixture("cellosaurus/search_idsy_molm13_20260917.json"),
    'idsy:"MOLM-13"': fixture("cellosaurus/search_idsy_molm_13_20260917.json"),
    'idsy:"MV4;11"': fixture("cellosaurus/search_idsy_mv4_11_semicolon_20260917.json"),
    'idsy:"KG1"': fixture("cellosaurus/search_idsy_kg1_20260917.json"),
    'idsy:"KG-1"': fixture("cellosaurus/search_idsy_kg_1_20260917.json"),
    'idsy:"NB4"': fixture("cellosaurus/search_idsy_nb4_20260917.json"),
    'dr:"ACH-000362"': fixture("cellosaurus/search_dr_ach_000362_20260917.json"),
    'dr:"SIDM00437"': fixture("cellosaurus/search_dr_sidm00437_20260917.json"),
    'dr:"CHEMBL3706573"': fixture("cellosaurus/search_dr_chembl3706573_20260917.json"),
    'dr:"MOLM13_950_2019"': fixture("cellosaurus/search_dr_molm13_950_2019_20260917.json"),
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


# Synthetic FHIR R4 patients for spec/entity/patient.md. No real or demo
# server data: every ID, date, and condition here is invented.
def fhir_condition(cid, text, status="active"):
    resource = {
        "resourceType": "Condition",
        "id": cid,
        "code": {"text": text},
        "onsetDateTime": "2020-01-01",
    }
    if status:
        resource["clinicalStatus"] = {"coding": [{"code": status}]}
    return resource


def fhir_bundle(conditions, next_url=None):
    bundle = {
        "resourceType": "Bundle",
        "type": "searchset",
        "entry": [{"resource": c, "search": {"mode": "match"}} for c in conditions],
    }
    if next_url:
        bundle["link"] = [{"relation": "next", "url": next_url}]
    return json.dumps(bundle).encode("utf-8")


FHIR_PATIENTS = {"SYNTH-PT-1", "SYNTH-PT-EMPTY", "SYNTH-PT-OFFSITE"}
FHIR_PAGE_TWO = "/fhir?_getpages=synth-pt-1&_getpagesoffset=1"
FHIR_SEARCH_PARAMS = ["gender", "birthdate", "_has"]
FHIR_NO_TOTAL_CONDITION = "http://example.org/synthetic|no-total"


def fhir_capability(params):
    return json.dumps({
        "resourceType": "CapabilityStatement",
        "rest": [{"mode": "server", "resource": [{
            "type": "Patient",
            "searchParam": [{"name": name, "type": "token"} for name in params],
        }]}],
    }).encode("utf-8")


def fhir_leaky_patient(pid):
    # A server that ignores _elements sends every field. BioMCP must drop them.
    return {
        "resourceType": "Patient", "id": pid, "gender": "female", "birthDate": "1970-01-01",
        "name": [{"family": "Synthleak", "given": ["Nameleak"]}],
        "address": [{"line": ["1 Addressleak Way"], "city": "Cityleak"}],
        "telecom": [{"system": "phone", "value": "555-0100-leak"}],
    }


def fhir_patient_search(params):
    # Ignores _elements and _count on purpose, and mixes in a Condition and a
    # Patient whose id breaks the FHIR id rule.
    entries = [
        {"resource": {"resourceType": "Condition", "id": "synth-condleak"}, "search": {"mode": "match"}},
        {"resource": fhir_leaky_patient("a/b"), "search": {"mode": "match"}},
    ] + [
        {"resource": fhir_leaky_patient(f"SYNTH-PT-S{n}"), "search": {"mode": "match"}}
        for n in range(1, 6)
    ]
    bundle = {"resourceType": "Bundle", "type": "searchset", "total": 7, "entry": entries}
    if params.get("_has:Condition:patient:code") == [FHIR_NO_TOTAL_CONDITION]:
        del bundle["total"]
    return json.dumps(bundle).encode("utf-8")


def fhir_response(parsed):
    path = parsed.path
    if path == "/fhir/metadata":
        return 200, fhir_capability(FHIR_SEARCH_PARAMS)
    if path == "/fhir-lacks-has/metadata":
        return 200, fhir_capability(["gender", "birthdate"])
    if path == "/fhir/Patient":
        return 200, fhir_patient_search(parse_qs(parsed.query))
    if path.startswith("/fhir/Patient/"):
        pid = path.rsplit("/", 1)[1]
        if pid in FHIR_PATIENTS:
            body = {"resourceType": "Patient", "id": pid, "gender": "female", "birthDate": "1970-01-01"}
            return 200, json.dumps(body).encode("utf-8")
        return 404, b'{"resourceType":"OperationOutcome","issue":[{"severity":"error","code":"not-found"}]}'
    params = parse_qs(parsed.query)
    if path == "/fhir/Condition":
        pid = params.get("patient", [""])[0]
        if pid == "SYNTH-PT-1":
            return 200, fhir_bundle([fhir_condition("synth-c1", "Synthetic condition one")], FHIR_PAGE_TWO)
        if pid == "SYNTH-PT-OFFSITE":
            return 200, fhir_bundle(
                [fhir_condition("synth-c3", "Synthetic condition three")],
                "http://offsite.invalid/fhir?page=2",
            )
        return 200, fhir_bundle([])
    if path == "/fhir" and params.get("_getpages") == ["synth-pt-1"]:
        return 200, fhir_bundle([fhir_condition("synth-c2", "Synthetic condition two", "resolved")])
    return None


def send(handler, status, body, content_type="application/json"):
    handler.send_response(status)
    handler.send_header("Content-Type", content_type)
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


class Handler(BaseHTTPRequestHandler):
    def do_HEAD(self):
        routed = resolve_request(self.path)
        if routed is None:
            send(self, 404, b'{"error":"unknown provider worker namespace"}')
            return
        parsed, request_target, request_log = routed
        with request_log.open("a", encoding="utf-8") as log:
            log.write(f"HEAD {request_target}\n")
        if (
            parsed.path == "/gencc/download/action/submissions-export-csv"
            and parse_qs(parsed.query) == {"format": ["new"]}
        ):
            self.send_response(200)
            self.send_header("Content-Type", "text/csv; charset=UTF-8")
            self.send_header("Content-Length", str(len(GENCC_ODC1)))
            self.send_header("ETag", GENCC_ETAG)
            self.send_header("Last-Modified", GENCC_LAST_MODIFIED)
            self.end_headers()
            return
        send(self, 404, b'{"error":"fixture route not found"}')

    def do_GET(self):
        routed = resolve_request(self.path)
        if routed is None:
            send(self, 404, b'{"error":"unknown provider worker namespace"}')
            return
        parsed, request_target, request_log = routed
        with request_log.open("a", encoding="utf-8") as log:
            conditional = ""
            if parsed.path == "/gencc/download/action/submissions-export-csv":
                conditional = (
                    f" If-None-Match={self.headers.get('If-None-Match', '')}"
                    f" If-Modified-Since={self.headers.get('If-Modified-Since', '')}"
                )
            log.write(f"GET {request_target}{conditional}\n")

        if parsed.path == "/healthz":
            send(self, 200, b'{"status":"ok"}')
            return
        if parsed.path == "/fhir" or parsed.path.startswith(("/fhir/", "/fhir-lacks-has/")):
            answer = fhir_response(parsed)
            if answer is not None:
                send(self, answer[0], answer[1], "application/fhir+json")
                return
        if parsed.path == "/mychem/v1/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            if query == "fixture-provider-failure":
                send(self, 503, b'{"error":"synthetic provider failure"}')
                return
            if query.startswith("ticket1151 hostile "):
                send(
                    self,
                    200,
                    json.dumps({
                        "total": 1,
                        "hits": [{
                            "_id": "DB-TICKET1151",
                            "_score": 10.0,
                            "drugbank": {
                                "id": "DB-TICKET1151",
                                "name": query,
                                "synonyms": [],
                            },
                        }],
                    }).encode("utf-8"),
                )
                return
            body = MYCHEM.get(query)
            if body is not None:
                send(self, 200, body)
                return
        if parsed.path == "/mygene/v3/query":
            params = parse_qs(parsed.query)
            query = params.get("q", [""])[0]
            offset = int(params.get("from", ["0"])[0])
            size = int(params.get("size", ["0"])[0])
            if query in {"(symbol:TOTAL50 OR alias:TOTAL50)", "(symbol:TOTAL51 OR alias:TOTAL51)"}:
                total = 50 if "TOTAL50" in query else 51
                send(self, 200, synthetic_gene_response(total, offset, size))
                return
            if query == FILTER_PATHWAY_QUERY:
                send(self, 200, FILTER_HITS)
                return
            if query == FILTER_GO_QUERY:
                send(self, 200, FILTER_HITS)
                return
            if "FILTERCASE" in query and "pathway." not in query and "go." not in query:
                send(self, 200, FILTER_HITS)
                return
            if query == "(symbol:HOSTILE OR alias:HOSTILE)":
                send(self, 200, HOSTILE)
                return
            if query in {r"ALK \(fusion\)", r"BRAF\:V600E", LUCENE_ESCAPED, "kinase"}:
                send(self, 200, b'{"total":0,"hits":[]}')
                return
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
        if (
            parsed.path == "/gencc/download/action/submissions-export-csv"
            and parse_qs(parsed.query) == {"format": ["new"]}
        ):
            if (
                self.headers.get("If-None-Match") == GENCC_ETAG
                and self.headers.get("If-Modified-Since") == GENCC_LAST_MODIFIED
            ):
                self.send_response(304)
                self.send_header("Content-Type", "text/csv; charset=UTF-8")
                self.send_header("Content-Length", "0")
                self.send_header("ETag", GENCC_ETAG)
                self.send_header("Last-Modified", GENCC_LAST_MODIFIED)
                self.end_headers()
            else:
                self.send_response(200)
                self.send_header("Content-Type", "text/csv; charset=UTF-8")
                self.send_header("Content-Length", str(len(GENCC_ODC1)))
                self.send_header("ETag", GENCC_ETAG)
                self.send_header("Last-Modified", GENCC_LAST_MODIFIED)
                self.end_headers()
                self.wfile.write(GENCC_ODC1)
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
        if parsed.path == "/chembl/status.json":
            send(self, 200, CHEMBL_STATUS)
            return
        if parsed.path == "/chembl/cell_line.json":
            # The join goes through the Cellosaurus accession alone. An
            # accession ChEMBL does not list is a normal zero-row result.
            accession = parse_qs(parsed.query).get("cellosaurus_id", [""])[0]
            send(self, 200, CHEMBL_CELL_LINES.get(accession, CHEMBL_CELL_LINE_EMPTY))
            return
        if parsed.path == "/chembl/assay.json":
            chembl_id = parse_qs(parsed.query).get("cell_chembl_id", [""])[0]
            page = CHEMBL_ASSAY_COUNTS.get(chembl_id)
            if page is not None:
                send(self, 200, page)
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
        if parsed.path == "/hpa/api/search_download.php":
            params = parse_qs(parsed.query)
            key = (params.get("search", [""])[0], params.get("columns", [""])[0])
            send(self, 200, HPA_CELL_RNA.get(key, b"[]"))
            return
        if parsed.path == "/cellosaurus/release-info":
            send(self, 200, CELLOSAURUS_RELEASE)
            return
        if parsed.path == "/cellosaurus/search/cell-line":
            # The window is answered by the query alone. An unlisted ac:, dr:,
            # idsy: or id: query is a normal zero-row result, never a failure.
            query = parse_qs(parsed.query).get("q", [""])[0]
            if query.startswith("id:("):
                first_name = query.split('"')[1] if '"' in query else ""
                send(self, 200, CELLOSAURUS_ID_BATCHES.get(first_name, CELLOSAURUS_EMPTY))
                return
            send(self, 200, CELLOSAURUS_SEARCHES.get(query, CELLOSAURUS_EMPTY))
            return
        if parsed.path.startswith("/cellosaurus/cell-line/"):
            # Matched on the path alone. The fields parameter is ignored, so one
            # recorded file answers every projection the command asks for.
            accession = parsed.path.rsplit("/", 1)[-1]
            record = CELLOSAURUS_RECORDS.get(accession)
            if record is None:
                send(self, 404, b'{"error":"cell line not found"}')
                return
            send(self, 200, record)
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
            query = parse_qsl(parsed.query, keep_blank_values=True)
            if query == [
                ("keyword", "melanoma"),
                ("include", "nct_id"),
                ("include", "nci_id"),
                ("include", "brief_title"),
                ("include", "current_trial_status"),
                ("include", "phase"),
                ("include", "diseases"),
                ("include", "lead_org"),
                ("include", "eligibility"),
                ("size", "1"),
                ("from", "0"),
            ]:
                send(self, 200, NCI_MELANOMA)
                return

        send(self, 404, b'{"error":"fixture route not found"}')

    def do_POST(self):
        routed = resolve_request(self.path)
        if routed is None:
            send(self, 404, b'{"error":"unknown provider worker namespace"}')
            return
        parsed, request_target, request_log = routed
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        with request_log.open("a", encoding="utf-8") as log:
            log.write(f"POST {request_target} {body.decode('utf-8')}\n")
        if parsed.path == "/fda-orphan/OOPD_Results.cfm":
            form = parse_qsl(body.decode("utf-8"), keep_blank_values=True)
            expected = [
                ("Product_name", form[0][1] if form else ""),
                ("sponsor_name", ""),
                ("Designation", ""),
                ("Designation_Start_Date", ""),
                ("Designation_End_Date", ""),
                ("Search_param", "DESDATE"),
                ("Output_Format", "Excel"),
                ("Sort_order", "GENERIC_NAME"),
                ("RecordsPerPage", "25"),
                ("newSearch", "Run Search"),
            ]
            if form == expected:
                send(self, 200, FDA_ORPHAN, "text/html; charset=UTF-8")
                return
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
        if parsed.path == "/pharmacodb/graphql":
            request = json.loads(body)
            query = request.get("query", "")
            variables = request.get("variables", {})
            if "cell_line(" in query:
                uid = variables.get("cellUID")
                name = variables.get("cellName")
                record = PHARMACODB_CELL_LINE_BY_UID.get(uid) if uid else PHARMACODB_CELL_LINE_BY_NAME.get(name)
                send(self, 200, record or PHARMACODB_CELL_LINE_MISSING)
                return
            if "compound(" in query:
                name = variables.get("compoundName", "")
                record = PHARMACODB_COMPOUNDS.get(name.lower())
                send(self, 200, record or PHARMACODB_COMPOUND_MISSING)
                return
            if "experiments(" in query:
                key = (variables.get("cellLineId"), variables.get("compoundId"))
                table = PHARMACODB_EXPERIMENTS if "AAC" in query else PHARMACODB_COUNTS
                record = table.get(key)
                if record is not None:
                    send(self, 200, record)
                    return
                send(self, 200, b'{"data":{"experiments":[]}}')
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
  cleanup_gencc_parent
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
  printf 'export BIOMCP_FDA_ORPHAN_BASE=%q\n' "$base_url/fda-orphan"
  printf 'export BIOMCP_CHEMBL_BASE=%q\n' "$base_url/chembl"
  printf 'export BIOMCP_OPENTARGETS_BASE=%q\n' "$base_url/opentargets/api/v4"
  printf 'export BIOMCP_QUICKGO_BASE=%q\n' "$base_url/quickgo/QuickGO/services"
  printf 'export BIOMCP_STRING_BASE=%q\n' "$base_url/string/api"
  printf 'export BIOMCP_HPA_BASE=%q\n' "$base_url/hpa"
  printf 'export BIOMCP_DGIDB_BASE=%q\n' "$base_url/dgidb/api"
  printf 'export BIOMCP_NIH_REPORTER_BASE=%q\n' "$base_url/nih/v2"
  printf 'export BIOMCP_CELLOSAURUS_BASE=%q\n' "$base_url/cellosaurus"
  printf 'export BIOMCP_PHARMACODB_BASE=%q\n' "$base_url/pharmacodb"
  printf 'export BIOMCP_KEGG_BASE=%q\n' "$base_url/kegg"
  printf 'export BIOMCP_REACTOME_BASE=%q\n' "$base_url/reactome/ContentService"
  printf 'export BIOMCP_WIKIPATHWAYS_BASE=%q\n' "$base_url/wikipathways"
  printf 'export BIOMCP_NCI_CTS_BASE=%q\n' "$base_url/nci/api/v2"
  printf 'export BIOMCP_GENCC_BASE=%q\n' "$base_url/gencc/download/action/submissions-export-csv?format=new"
  printf 'export BIOMCP_GENCC_DIR=%q\n' "$gencc_dir"
  printf 'export BIOMCP_GENCC_FIXTURE_PARENT=%q\n' "$gencc_parent"
  printf 'export BIOMCP_GENCC_FIXTURE_TMP_ROOT=%q\n' "$gencc_tmp_root"
  printf 'export NCI_API_KEY=%q\n' 'fixture-nci-key'
  printf 'export BIOMCP_EMA_DIR=%q\n' "$ema_dir"
  printf 'export BIOMCP_WHO_DIR=%q\n' "$who_dir"
  printf 'export BIOMCP_WHO_IVD_DIR=%q\n' "$who_ivd_dir"
  printf 'export BIOMCP_GTR_DIR=%q\n' "$gtr_dir"
  printf 'export BIOMCP_PROVIDER_CONTRACT_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_PROVIDER_CONTRACT_ROOT=%q\n' "$fixture_root"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_PROVIDER_CONTRACT_READY_FILE=%q\n' "$ready_file"
  printf 'export BIOMCP_PROVIDER_CONTRACT_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "provider-contract" "$fixture_root" "$server_pid" "BIOMCP_PROVIDER_CONTRACT" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
