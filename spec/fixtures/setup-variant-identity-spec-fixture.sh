#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ownership_helper="$script_dir/routine-fixture-ownership.sh"
# shellcheck source=fixture-supervisor.sh
source "$script_dir/fixture-supervisor.sh"

workspace_root="${1:-$PWD}"
cache_dir="$workspace_root/.cache"
env_file="$cache_dir/spec-variant-identity-env"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cleanup_script="$script_dir/cleanup-variant-identity-spec-fixture.sh"

mkdir -p "$cache_dir"
bash "$cleanup_script" "$workspace_root"
recover_fixture_orphans "$cache_dir" "variant-identity" "spec-variant-identity."

fixture_root="$(mktemp -d "$cache_dir/spec-variant-identity.XXXXXX")"
owner_arg="$(bash "$ownership_helper" new-owner "variant-identity" "$fixture_root")"
ready_file="$fixture_root/base-url"
server_log="$fixture_root/server.log"
request_log="$fixture_root/request.log"
server_pid_file="$fixture_root/server-pid"
: >"$request_log"

prepare_fixture_supervisor_owner
start_fixture_supervisor "variant-identity" "$cache_dir" "$fixture_root" "spec-variant-identity." "$server_pid_file" \
  python3 - "$workspace_root" "$ready_file" "$request_log" "$owner_arg" <<'PY' >"$server_log" 2>&1 &
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, unquote, urlparse
import json
import sys
import time

ROOT = Path(sys.argv[1])
READY = Path(sys.argv[2])
REQUEST_LOG = Path(sys.argv[3])
SEARCH_RESPONSE = json.loads(
    (ROOT / "testdata/sources/myvariant/search_brca1_contradictory_protein.json").read_text(
        encoding="utf-8"
    )
)
BRAF_MISSENSE_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_missense_20260805.json").read_bytes()
BRAF_REVEL_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_revel_20260805.json").read_bytes()
BRAF_V600E_RESPONSE = (ROOT / "testdata/sources/myvariant/search_braf_v600e_20260806.json").read_bytes()
BRAF_V600E_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_braf_v600e_grch38_20260806.json").read_bytes()
BRAF_V600E_GRCH37_RESPONSE = (ROOT / "testdata/sources/myvariant/get_braf_v600e.json").read_bytes()
GRID1_GRCH37_RESPONSE = (ROOT / "testdata/sources/myvariant/get_grid1_grch37_20260806.json").read_bytes()
GRID1_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_grid1_grch38_20260806.json").read_bytes()
PTEN_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_pten_grch38_20260806.json").read_bytes()
PTEN_DELETION_GRCH38_RESPONSE = (ROOT / "testdata/sources/myvariant/get_pten_deletion_grch38_20260806.json").read_bytes()
SMARCA4_REPEAT_RESPONSE = json.dumps({
    "_id": "chr19:g.11106928AAG[1]",
    "dbsnp": {"rsid": "rs876657378"},
    "clinvar": {"gene": {"symbol": "SMARCA4"}},
}).encode("utf-8")
RS334_GRCH37_RESPONSE = {
    "_id": "chr11:g.5248232T>A",
    "dbsnp": {"rsid": "rs334"},
    "clinvar": {"gene": {"symbol": "HBB"}},
}
RS334_GRCH38_RESPONSE = {
    "_id": "chr11:g.5227002T>A",
    "dbsnp": {"rsid": "rs334"},
    "clinvar": {"gene": {"symbol": "HBB"}},
}
MYD88_L265P_RESPONSE = (ROOT / "testdata/sources/myvariant/search_myd88_l265p_20260806.json").read_bytes()
TP53_G105S_RESPONSE = (ROOT / "testdata/sources/myvariant/search_tp53_g105s_20261003.json").read_bytes()
# Ticket 1297: recorded gene+protein queries whose alias spans several
# genomic variants. Only the DICER1 ClinVar record names one of its three.
DICER1_M1483I_RESPONSE = (ROOT / "testdata/sources/myvariant/query_dicer1_m1483i_20261006.json").read_bytes()
EGFR_M766I_RESPONSE = (ROOT / "testdata/sources/myvariant/query_egfr_m766i_20261006.json").read_bytes()
# Ticket 2018: recorded KRAS codon-12 queries. rs121913529 names three alleles
# (G12A, G12D, G12V); each gene+protein query names exactly one of them.
KRAS_G12A_RESPONSE = (ROOT / "testdata/sources/myvariant/query_kras_g12a_20261007.json").read_bytes()
KRAS_G12D_RESPONSE = (ROOT / "testdata/sources/myvariant/query_kras_g12d_20261007.json").read_bytes()
KRAS_G12V_RESPONSE = (ROOT / "testdata/sources/myvariant/query_kras_g12v_20261007.json").read_bytes()
RS121913529_RESPONSE = (ROOT / "testdata/sources/myvariant/query_rsid_rs121913529_20261007.json").read_bytes()
CLINVAR_428884_XML = (ROOT / "testdata/sources/ncbi_efetch/clinvar_428884_20261003.xml").read_bytes()
# Ticket 1291: the switch never reproduced live (the recorded
# reproduction runs' efetch calls all answered within 1.4 s), so this
# synthetic hold replays the code-confirmed trigger: an NCBI efetch
# answer slower than the optional-enrichment deadline. The ClinVar
# section must name the deadline miss and label the served
# MyVariant.info copy as degraded with its newest evaluation date.
CLINVAR_TIMEOUT_HIT = {
    "_id": "chr17:g.7676154G>A",
    "dbnsfp": {"genename": "TP53", "hgvsp": "p.R273H"},
    "clinvar": {
        "gene": {"symbol": "TP53"},
        "variant_id": 1290630,
        "rcv": [{
            "accession": "RCV000030704",
            "clinical_significance": "Pathogenic",
            "last_evaluated": "2021-03-11",
            "number_submitters": 3,
            "preferred_name": "NM_000546.6(TP53):c.818G>A (p.Arg273His)",
            "review_status": "reviewed by expert panel",
        }],
    },
}
H3F3A_K28M_HIT = {
    "_id": "chr1:g.226252135A>T",
    "dbnsfp": {
        "genename": "H3F3A",
        "hgvsp": "p.K28M",
        "hgvsc": "NM_002107.4:c.83A>T",
    },
}
H3F3A_POSITION_HITS = [
    {
        "_id": f"chr1:g.{position}A>T",
        "dbnsfp": {"genename": "H3F3A", "hgvsp": f"p.K{position}M"},
    }
    for position in (19, 28, 37, 57)
]
# Ticket 1301: recorded shape for a gene-and-condition search whose free-text
# form now routes the gene-symbol first token. Three rows, same as the
# explicit -g/--condition form.
SCN5A_BRUGADA_HITS = [
    {
        "_id": f"chr3:g.{position}G>A",
        "dbnsfp": {"genename": "SCN5A", "hgvsp": change},
    }
    for position, change in (
        (38592622, "p.R1193Q"),
        (38618015, "p.R2255W"),
        (38553661, "p.R367H"),
    )
]
BRUGADA_CONDITION_HIT = {
    "_id": "chr12:g.50879715G>A",
    "dbnsfp": {"genename": "PKP2", "hgvsp": "p.Q63K"},
}
HSD17B4_TRANSCRIPT_HIT = {
    "_id": "chr5:g.118860951A>G",
    "dbnsfp": {
        "genename": "HSD17B4",
        "hgvsp": ["p.His515Arg", "p.His540Arg"],
        "hgvsc": ["c.1544A>G", "c.1619A>G"],
    },
    "clinvar": {"rcv": {"preferred_name": "NM_000414.3(HSD17B4):c.1544A>G (p.His515Arg)"}},
    "snpeff": {"ann": [
        {"feature_id": "NM_001199291.2", "genename": "HSD17B4", "hgvs_c": "c.1619A>G", "hgvs_p": "p.His540Arg"},
        {"feature_id": "NM_000414.3", "genename": "HSD17B4", "hgvs_c": "c.1544A>G", "hgvs_p": "p.His515Arg"},
        {"feature_id": "XM_011512026.2", "genename": "HSD17B4", "hgvs_c": "c.1616A>G", "hgvs_p": "p.His539Arg"},
        {"feature_id": "NM_001199292.2", "genename": "HSD17B4", "hgvs_c": "c.1562A>G", "hgvs_p": "p.His521Arg"},
        {"feature_id": "XM_017009363.1", "genename": "HSD17B4", "hgvs_c": "c.1544A>G", "hgvs_p": "p.His515Arg"},
    ]},
}
HOSTILE_MATCH_GENE = "HOSTILE\n\r\t\x06\x85\x1b[32m\u202e|`MATCH"
HOSTILE_MATCH_PROTEIN = "p.His2Arg\n\r\t\x08\x85\x1b[33m\u2066|```MATCH"
HOSTILE_MATCH_QUERY = 'dbnsfp.hgvsp:"' + "".join(
    "\\" + char if char in '\\+-!(){}[]^"~*?:/&|' else char
    for char in HOSTILE_MATCH_PROTEIN
) + '"'
HOSTILE_TRANSCRIPT_HIT = {
    "_id": "chr2:g.2A>G",
    "dbnsfp": {"genename": HOSTILE_MATCH_GENE, "hgvsp": HOSTILE_MATCH_PROTEIN},
    "snpeff": {"ann": [
        {
            "feature_id": "NM_1\n\r\t\x01\x85\x1b[31m\u202e# injected|`",
            "genename": "DISPLAY\n\r\t\x02\x85\x1b[31m\u2066|`",
            "hgvs_c": "c.1A>G\n\r\t\x03\x85\x1b]0;bad\x07\u2069|- item``",
            "hgvs_p": "p.A1V\n\r\t\x04\x85\x1bPbad\x1b\\\u202e|```",
        },
        {
            "feature_id": "MATCH\n\r\t\x05\x85\x1b[31m\u202e|`",
            "genename": HOSTILE_MATCH_GENE,
            "hgvs_c": "c.2A>G\n\r\t\x07\x85\x1b]0;bad\u2069|``",
            "hgvs_p": HOSTILE_MATCH_PROTEIN,
        },
        {},
    ]},
}
CANCERHOTSPOTS_RESPONSES = {
    "/api/hotspots/single/byGene/BRAF": (ROOT / "testdata/sources/cancerhotspots/by_gene_braf_20260805.json").read_bytes(),
    "/api/hotspots/single/byGene/MYD88": (ROOT / "testdata/sources/cancerhotspots/by_gene_myd88_20260805.json").read_bytes(),
    "/api/hotspots/single/byGene/NOTAREALGENE664": (ROOT / "testdata/sources/cancerhotspots/by_gene_empty_20260805.json").read_bytes(),
}
# Ticket 1292: recorded transcript-normalization and ClinVar-alias responses
# for the get-variant input-form table. Mutalyzer refuses the intronic
# deletion range (HTTP 422), so the ClinVar coding-alias search carries it.
MUTALYZER_NORMALIZE_RESPONSES = {
    "NM_000249.4:c.678-14_678-3del": (
        422,
        (ROOT / "testdata/sources/mutalyzer/normalize_nm_000249.4_c.678-14_678-3del_20261006.json").read_bytes(),
    ),
    "NM_177438.3:c.4449G>A": (
        200,
        (ROOT / "testdata/sources/mutalyzer/normalize_nm_177438.3_c.4449G_to_A_20261006.json").read_bytes(),
    ),
}
VARIANTVALIDATOR_NORMALIZE_RESPONSES = {
    "NM_000249.4:c.678-14_678-3del": (
        200,
        (ROOT / "testdata/sources/variantvalidator/normalize_nm_000249.4_c.678-14_678-3del_20261006.json").read_bytes(),
    ),
    "NM_177438.3:c.4449G>A": (
        200,
        (ROOT / "testdata/sources/variantvalidator/normalize_nm_177438.3_c.4449G_to_A_20261006.json").read_bytes(),
    ),
}
CAR_ALLELE_RESPONSES = {
    "NM_000249.4:c.678-14_678-3del": (
        200,
        (ROOT / "testdata/sources/clingen_allele_registry/allele_nm_000249.4_c.678-14_678-3del_20261006.json").read_bytes(),
    ),
    "NM_177438.3:c.4449G>A": (
        200,
        (ROOT / "testdata/sources/clingen_allele_registry/allele_nm_177438.3_c.4449G_to_A_20261006.json").read_bytes(),
    ),
}
CLINVAR_CODING_ALIAS_RESPONSES = {
    'clinvar.hgvs.coding:"NM_000249.4\\:c.678\\-14_678\\-3del"': (
        ROOT / "testdata/sources/myvariant/query_clinvar_coding_nm_000249.4_c.678-14_678-3del_20261006.json"
    ).read_bytes(),
    'clinvar.hgvs.coding:"NM_177438.3\\:c.4449G>A"': (
        ROOT / "testdata/sources/myvariant/query_clinvar_coding_nm_177438.3_c.4449G_to_A_20261006.json"
    ).read_bytes(),
    "clinvar.variant_id:577152": (
        ROOT / "testdata/sources/myvariant/query_clinvar_variant_id_577152_20261006.json"
    ).read_bytes(),
    'dbnsfp.genename:EGFR AND dbnsfp.hgvsp:"p.E746_A750del"': (
        ROOT / "testdata/sources/myvariant/query_egfr_e746_a750del_20261006.json"
    ).read_bytes(),
}


def send_json(handler, status, payload):
    body = payload if isinstance(payload, bytes) else json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def send_xml(handler, status, payload):
    handler.send_response(status)
    handler.send_header("Content-Type", "application/xml")
    handler.send_header("Content-Length", str(len(payload)))
    handler.end_headers()
    handler.wfile.write(payload)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"GET {self.path}\n")

        if parsed.path == "/healthz":
            send_json(self, 200, {"status": "ok"})
            return
        if parsed.path.startswith("/api/normalize/"):
            description = unquote(parsed.path[len("/api/normalize/") :])
            if description in MUTALYZER_NORMALIZE_RESPONSES:
                status, body = MUTALYZER_NORMALIZE_RESPONSES[description]
                send_json(self, status, body)
                return
            send_json(self, 400, {"error": "unexpected fixture normalize request"})
            return
        if (
            parsed.path.startswith("/VariantValidator/variantvalidator/GRCh38/")
            and parsed.path.endswith("/all")
        ):
            description = unquote(parsed.path).split("/GRCh38/", 1)[1][
                : -len("/all")
            ]
            if description in VARIANTVALIDATOR_NORMALIZE_RESPONSES:
                status, body = VARIANTVALIDATOR_NORMALIZE_RESPONSES[description]
                send_json(self, status, body)
                return
            send_json(self, 400, {"error": "unexpected fixture validate request"})
            return
        if parsed.path == "/allele":
            hgvs = parse_qs(parsed.query).get("hgvs", [""])[0]
            if hgvs in CAR_ALLELE_RESPONSES:
                status, body = CAR_ALLELE_RESPONSES[hgvs]
                send_json(self, status, body)
                return
            send_json(self, 400, {"error": "unexpected fixture allele request"})
            return
        if parsed.path == "/efetch.fcgi":
            params = parse_qs(parsed.query)
            if params.get("db") == ["clinvar"] and params.get("id") == ["428884"]:
                send_xml(self, 200, CLINVAR_428884_XML)
                return
            if params.get("db") == ["clinvar"] and params.get("id") == ["1290630"]:
                # Ticket 1291: a synthetic hold, not a recorded exchange — no
                # provider control forces a real eight-second NCBI hold. The
                # optional-enrichment deadline fires before this answer lands.
                time.sleep(20)
                send_xml(self, 200, CLINVAR_428884_XML)
                return
            send_json(self, 404, {"error": "fixture efetch record not found"})
            return
        if parsed.path == "/v1/variant/chr7:g.140753336A%3ET":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, BRAF_V600E_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr7:g.140453136A%3ET":
            if parse_qs(parsed.query).get("assembly") == ["hg19"]:
                send_json(self, 200, BRAF_V600E_GRCH37_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87933119A%3EC":
            if parse_qs(parsed.query).get("assembly") == ["hg19"]:
                send_json(self, 200, GRID1_GRCH37_RESPONSE)
                return
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, GRID1_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87925512G%3EA":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, PTEN_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr10:g.87925512del":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, PTEN_DELETION_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/v1/variant/chr19:g.11106928AAG%5B1%5D":
            send_json(self, 200, SMARCA4_REPEAT_RESPONSE)
            return
        if parsed.path == "/v1/variant/chr11:g.5227002T%3EA":
            if parse_qs(parsed.query).get("assembly") == ["hg38"]:
                send_json(self, 200, RS334_GRCH38_RESPONSE)
                return
            send_json(self, 404, {"code": 404, "success": False, "error": "Not Found."})
            return
        if parsed.path == "/refsnp/334":
            send_json(self, 200, {
                "primary_snapshot_data": {"placements_with_allele": [{
                    "is_ptlp": True,
                    "placement_annot": {"seq_id_traits_by_assembly": [{
                        "assembly_name": "GRCh38.p14", "is_chromosome": True
                    }]},
                    "alleles": [{"allele": {"spdi": {
                        "seq_id": "NC_000011.10", "position": 5227001,
                        "deleted_sequence": "T", "inserted_sequence": "A"
                    }}}],
                }]}
            })
            return

        if parsed.path == "/v1/query":
            query = parse_qs(parsed.query).get("q", [""])[0]
            if query in CLINVAR_CODING_ALIAS_RESPONSES:
                send_json(self, 200, CLINVAR_CODING_ALIAS_RESPONSES[query])
                return
            expected_proteins = ('dbnsfp.hgvsp:"p.M1783I"', 'dbnsfp.hgvsp:"p.M16I"')
            if query == "dbnsfp.genename:H3\\-3A":
                send_json(self, 200, {"total": 0, "hits": []})
                return
            if query == "dbnsfp.genename:H3F3A":
                send_json(self, 200, {"total": 1156, "hits": [H3F3A_K28M_HIT]})
                return
            if query == "dbnsfp.genename:RB1":
                send_json(self, 200, {
                    "total": 1,
                    "hits": [{
                        "_id": "chr13:g.1C>T",
                        "dbnsfp": {"genename": "RB1", "hgvsp": "p.R1W"},
                    }],
                })
                return
            if query in (
                'dbnsfp.genename:RB1 AND dbnsfp.hgvsp:"p.Q999X"',
                'dbnsfp.genename:RB1 AND dbnsfp.hgvsp:"p.Q999\\*"',
                "dbnsfp.genename:RB1 AND dbnsfp.hgvsp:p.Q**",
            ):
                send_json(self, 200, {"total": 0, "hits": []})
                return
            if query == 'dbnsfp.genename:H3F3A AND dbnsfp.hgvsp:"p.K27M"':
                send_json(self, 200, {"total": 0, "hits": []})
                return
            if query == 'dbnsfp.genename:H3F3A AND dbnsfp.hgvsp:"p.K28M"':
                send_json(self, 200, {"total": 1, "hits": [H3F3A_K28M_HIT]})
                return
            if query == "dbnsfp.genename:H3F3A AND dbnsfp.hgvsp:p.K*M":
                send_json(self, 200, {"total": 4, "hits": H3F3A_POSITION_HITS})
                return
            if "dbnsfp.genename:HSD17B4" in query:
                send_json(self, 200, {"total": 1, "hits": [HSD17B4_TRANSCRIPT_HIT]})
                return
            if query == HOSTILE_MATCH_QUERY:
                send_json(self, 200, {"total": 1, "hits": [HOSTILE_TRANSCRIPT_HIT]})
                return
            if query == "dbnsfp.genename:NOTAREALGENE1091":
                send_json(self, 200, {"total": 0, "hits": []})
                return
            # Ticket 1301: routed gene-and-condition search and the condition
            # searches a refused or non-gene phrase falls back to.
            if query == 'dbnsfp.genename:SCN5A AND clinvar.rcv.conditions.name:"Brugada"':
                send_json(self, 200, {"total": 3, "hits": SCN5A_BRUGADA_HITS})
                return
            if query == 'clinvar.rcv.conditions.name:"BRUGADA syndrome"':
                send_json(self, 200, {"total": 0, "hits": []})
                return
            if query == 'clinvar.rcv.conditions.name:"brugada syndrome"':
                send_json(self, 200, {"total": 1, "hits": [BRUGADA_CONDITION_HIT]})
                return
            if query == "dbnsfp.genename:H3F3A AND cadd.phred:[99 TO *]":
                send_json(self, 200, {"total": 0, "hits": []})
                return
            if query == "cadd.phred:[99 TO *]":
                send_json(self, 200, {
                    "total": 1,
                    "hits": [{
                        "_id": "chr2:g.1091A>T",
                        "dbnsfp": {"genename": "OTHER", "hgvsp": "p.A1V"},
                    }],
                })
                return
            if query == "dbsnp.rsid:rs876657378":
                send_json(self, 200, {"total": 1, "hits": [json.loads(SMARCA4_REPEAT_RESPONSE)]})
                return
            if query == "dbsnp.rsid:rs334":
                send_json(self, 200, {"total": 1, "hits": [RS334_GRCH37_RESPONSE]})
                return
            if "dbnsfp.genename:BRCA1" in query and any(
                protein in query for protein in expected_proteins
            ):
                send_json(self, 200, SEARCH_RESPONSE)
                return
            if (
                "dbnsfp.genename:BRAF" in query
                and "snpeff.ann.effect:*missense_variant*" in query
            ):
                send_json(self, 200, BRAF_MISSENSE_RESPONSE)
                return
            if "dbnsfp.genename:BRAF" in query and "_exists_:dbnsfp.revel.score" in query:
                send_json(self, 200, BRAF_REVEL_RESPONSE)
                return
            if "dbnsfp.genename:BRAF" in query and 'dbnsfp.hgvsp:"p.V600E"' in query:
                send_json(self, 200, BRAF_V600E_RESPONSE)
                return
            if "dbnsfp.genename:MYD88" in query and 'dbnsfp.hgvsp:"p.L265P"' in query:
                send_json(self, 200, MYD88_L265P_RESPONSE)
                return
            if "dbnsfp.genename:TP53" in query and 'dbnsfp.hgvsp:"p.G105S"' in query:
                send_json(self, 200, TP53_G105S_RESPONSE)
                return
            if "dbnsfp.genename:TP53" in query and 'dbnsfp.hgvsp:"p.R273H"' in query:
                send_json(self, 200, {"total": 1, "hits": [CLINVAR_TIMEOUT_HIT]})
                return
            if query == 'dbnsfp.genename:DICER1 AND dbnsfp.hgvsp:"p.M1483I"':
                send_json(self, 200, json.loads(DICER1_M1483I_RESPONSE))
                return
            if query == 'dbnsfp.genename:EGFR AND dbnsfp.hgvsp:"p.M766I"':
                send_json(self, 200, json.loads(EGFR_M766I_RESPONSE))
                return
            if query == 'dbnsfp.genename:KRAS AND dbnsfp.hgvsp:"p.G12A"':
                send_json(self, 200, KRAS_G12A_RESPONSE)
                return
            if query == 'dbnsfp.genename:KRAS AND dbnsfp.hgvsp:"p.G12D"':
                send_json(self, 200, KRAS_G12D_RESPONSE)
                return
            if query == 'dbnsfp.genename:KRAS AND dbnsfp.hgvsp:"p.G12V"':
                send_json(self, 200, KRAS_G12V_RESPONSE)
                return
            if query == "dbsnp.rsid:rs121913529":
                send_json(self, 200, RS121913529_RESPONSE)
                return
            send_json(self, 400, {"error": "unexpected fixture query"})
            return

        if parsed.path in CANCERHOTSPOTS_RESPONSES:
            send_json(self, 200, CANCERHOTSPOTS_RESPONSES[parsed.path])
            return

        send_json(self, 404, {"error": "fixture path not found"})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        with REQUEST_LOG.open("a", encoding="utf-8") as log:
            log.write(f"POST {self.path} {body.decode('utf-8')}\n")
        if b'"variantId":"11-5227002-T-A"' in body:
            send_json(self, 200, {"data": {"variant": {
                "variant_id": "11-5227002-T-A",
                "exome": {
                    "ac": 2, "an": 2000, "homozygote_count": 0,
                    "hemizygote_count": 0, "filters": [], "faf95": None,
                    "populations": [],
                },
                "genome": None,
            }}})
            return
        send_json(self, 400, {"error": "unexpected fixture query"})

    def log_message(self, format, *args):
        return


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
READY.write_text(f"http://127.0.0.1:{server.server_port}\n", encoding="utf-8")
server.serve_forever()
PY
supervisor_pid=$!
for _ in $(seq 1 50); do test -s "$server_pid_file" && break; kill -0 "$supervisor_pid" 2>/dev/null || break; sleep .1; done
test -s "$server_pid_file"
server_pid="$(<"$server_pid_file")"
cleanup_partial_setup() {
  kill -TERM -- "-$server_pid" 2>/dev/null || true
  wait "$supervisor_pid" 2>/dev/null || true
  rm -rf "$fixture_root"
}
trap cleanup_partial_setup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for _ in $(seq 1 50); do
  if [[ -s "$ready_file" ]]; then
    break
  fi
  if ! kill -0 "$server_pid" 2>/dev/null; then
    cat "$server_log" >&2
    exit 1
  fi
  sleep 0.1
done

test -s "$ready_file"
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
  printf 'export BIOMCP_MYVARIANT_BASE=%q\n' "$base_url/v1"
  printf 'export BIOMCP_DBSNP_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_GNOMAD_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CANCERHOTSPOTS_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CLINVAR_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_MUTALYZER_BASE_URL=%q\n' "$base_url/api"
  printf 'export BIOMCP_VARIANTVALIDATOR_BASE_URL=%q\n' "$base_url"
  printf 'export BIOMCP_CLINGEN_CAR_BASE=%q\n' "$base_url"
  printf 'export BIOMCP_CACHE_MODE=off\n'
  printf 'export BIOMCP_VARIANT_IDENTITY_REQUEST_LOG=%q\n' "$request_log"
} >"$env_file"

bash "$ownership_helper" write "$workspace_root" "variant-identity" "$fixture_root" "$server_pid" "BIOMCP_VARIANT_IDENTITY" "$owner_arg" >/dev/null
trap - EXIT INT TERM HUP
printf '%s\n' "$fixture_root"
