#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
repo_root="$(cd "$repo_root" && pwd)"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/biomcp-g5-identity.XXXXXX")"
trap 'rm -rf "$fixture_root"' EXIT
ready="$fixture_root/ready"
mode="$fixture_root/mode"
printf 'normal\n' >"$mode"

uv run --no-sync python - "$ready" "$mode" <<'PY' >"$fixture_root/server.log" 2>&1 8>&- &
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ready = Path(sys.argv[1])
mode = Path(sys.argv[2])
# The frozen rows deliberately include a true positive and collision(s) for APC.
# They are provider response fixtures, not clinical classifications.
rows = {
    "APC": [("12901799", "APC positive"), ("31749828", "APC retrieval collision"), ("90000001", "APC mixed observation")],
    "ATM": [("32918381", "ATM positive")],
    "PALB2": [("39999518", "PALB2 positive")],
    "MLH1": [("20864636", "MLH1 positive")],
    "BRCA1": [("33656647", "BRCA1 retrieval collision"), ("90000003", "BRCA1 unlinked fixture article")],
    "PTEN": [("90000002", "PTEN unavailable verification")],
    "TP53": [("24376681", "TP53 retrieval collision")],
}
# Minimized PubTator3-shaped captures. These are provider facts, not clinical classifications.
# Each tuple is (gene symbol, exact provider HGVS, NCBI Gene ID).
passages = {
    "12901799": [("APC", "p.Arg283Ter", 324)],
    "31749828": [("TP53", "c.847C>T", 7157)],
    "90000001": [("APC", "p.Arg283Ter", 324), ("APC", "p.Arg283Gln", 324)],
    "32918381": [("ATM", "c.1066-6T>G", 472)],
    "39999518": [("PALB2", "c.3350+5G>A", 79728)],
    "20864636": [("MLH1", "p.Leu749Pro", 4292)],
    "33656647": [("BRCA1", "c.788G>T", 672)],
    "90000003": [("BRCA1", "c.788G>T", 672)],
    "24376681": [("NKX2-5", "c.356C>A", 1482)],
    "90000004": [("BRAF", "p.Val600Glu", 673)],
    "90000005": [("BRAF", "p.Val600Glu", 673)],
    "90000006": [("BRAF", "p.Val600Glu", 673)],
}

def send(h, status, value):
    body = json.dumps(value).encode()
    h.send_response(status); h.send_header("Content-Type", "application/json"); h.send_header("Content-Length", str(len(body))); h.end_headers(); h.wfile.write(body)

def article(pmid, title):
    return {"_id": pmid, "pmid": pmid, "title": title, "journal": "Frozen fixture", "date": "2024-01-01", "score": 1}

# Opaque fixture-only tokens exercise BioMCP aggregation; they are not registry CAids.
car_ids = {
    "NM_000038.6:c.847C>T": "CA900000000001", "NC_000005.10:g.112815507C>T": "CA900000000001",
    "NM_000051.4:c.1066-6T>G": "CA900000000002", "NC_000011.10:g.108248927T>G": "CA900000000002",
    "NM_007294.4:c.2428A>T": "CA900000000003", "NC_000017.11:g.43093103T>A": "CA900000000003",
    "NM_000249.4:c.2246T>C": "CA900000000004", "NC_000003.12:g.37050628T>C": "CA900000000004",
    "NM_024675.4:c.3350+5G>A": "CA900000000005", "NC_000016.10:g.23607859C>T": "CA900000000005",
    "NM_000314.8:c.517C>T": "CA900000000006", "NC_000010.11:g.87952142C>T": "CA900000000006",
    "NM_000546.6:c.356C>G": "CA900000000007", "NC_000017.11:g.7676013G>C": "CA900000000007",
    "NM_004333.6:c.1799T>A": "CA900000000008", "NC_000007.13:g.140453136A>T": "CA900000000008",
}

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def do_GET(self):
        parsed = urlparse(self.path); query = parse_qs(parsed.query); path = parsed.path
        if path == "/allele":
            caid = car_ids.get(query.get("hgvs", [""])[0])
            if caid:
                return send(self, 200, {"@id": f"https://fixture.invalid/{caid}"})
            return send(self, 200, {"@id": "_:CA"})
        if path == "/v1/query": return send(self, 200, {"total": 0, "hits": []})
        if path.startswith("/v1/variant/"): return send(self, 404, {"error": "not found"})
        if path == "/entity/autocomplete/": return send(self, 200, [])
        if path == "/search/":
            text = query.get("text", [""])[0]
            found = next((gene for gene in rows if gene in text), None)
            values = [article(*value) for value in rows.get(found, [])]
            if mode.read_text().strip() == "reordered":
                values.reverse()
            return send(self, 200, {"results": values, "count": len(values), "total_pages": 1, "current": 1, "page_size": 25})
        if path == "/publications/export/biocjson":
            pmid = query.get("pmids", [""])[0]
            if pmid == "90000002": return send(self, 503, {"error": "fixture outage"})
            pairs = passages.get(pmid, [])
            # BRCA1 has the exact article-wide gene/allele pair but only an unrelated
            # Association. It must not be upgraded from co-occurrence to proof.
            typed_linkage = pmid != "90000003"
            docs = [{
                "id": pmid,
                "pmid": int(pmid),
                "passages": [{
                    "infons": {"type": "abstract"},
                    "text": f"{gene} {allele} frozen content.",
                    "annotations": [
                        {
                            "id": f"gene-{index}",
                            "text": "provider text must not be used as proof",
                            "infons": {"type": "Gene", "name": gene, "identifier": str(gene_id), "normalized_id": gene_id},
                        },
                        {
                            "id": f"variant-{index}",
                            "text": "provider text must not be used as proof",
                            "infons": ({"type": "Variant", "hgvs": allele, "identifier": f"Variant:{allele}"}
                                if not typed_linkage else {
                                    "type": "Variant", "hgvs": allele, "gene_id": gene_id, "gene_ids": [gene_id],
                                    "identifier": f"Variant:{allele};CorrespondingGene:{gene_id}",
                                }),
                        },
                    ],
                } for index, (gene, allele, gene_id) in enumerate(pairs, start=1)],
                # Association is deliberately unrelated proof: CorrespondingGene facts above
                # must carry confirmation, not arbitrary BioC relation membership.
                "relations": [{
                    "id": f"association-{index}",
                    "infons": {"type": "Association"},
                    "nodes": [
                        {"refid": f"gene-{index}", "role": "subject"},
                        {"refid": f"variant-{index}", "role": "object"},
                    ],
                } for index, _ in enumerate(pairs, start=1)],
            }]
            if pmid == "12901799":
                # Identity anomalies are diagnostic only. The canonical duplicate and
                # document reordering exercise deterministic expected-PMID aggregation.
                docs.extend([
                    docs[0],
                    {"id": "99999999", "pmid": 99999999, "passages": []},
                    {"id": pmid, "pmid": 99999999, "passages": []},
                    {"pmid": 99999999, "passages": []},
                ])
            if pmid == "90000003":
                docs.append({"pmid": 99999999, "passages": []})
            if mode.read_text().strip() == "reordered":
                docs.reverse()
            return send(self, 200, {"PubTator3": docs})
        # This provider-shaped deep page sequence repeats one valid PMID. It exercises
        # pagination work without inventing an internal shortcut: PubMed still returns
        # ESearch IDs and matching ESummary records that BioMCP must deduplicate.
        if path.endswith("/esearch.fcgi"):
            if mode.read_text().strip() == "deep-discovery":
                return send(self, 200, {"esearchresult": {"idlist": ["90000004", "90000005", "90000006"], "count": "100"}})
            return send(self, 200, {"esearchresult": {"idlist": [], "count": "0"}})
        if path.endswith("/esummary.fcgi"):
            if mode.read_text().strip() == "deep-discovery":
                pmids = query.get("id", [""])[0].split(",")
                return send(self, 200, {"result": {"uids": pmids, **{pmid: {"uid": pmid, "title": "BRAF V600E frozen identity article"} for pmid in pmids}}})
            return send(self, 200, {"result": {"uids": []}})
        if path in {"/sentences/", "/passages/"}: return send(self, 200, [])
        return send(self, 200, {"results": [], "data": [], "total": 0, "resultList": {"result": []}})

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready.write_text(f"http://127.0.0.1:{server.server_port}")
server.serve_forever()
PY
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true; rm -rf "$fixture_root"' EXIT
for _ in $(seq 1 100); do test -s "$ready" && break; sleep 0.05; done
base_url="$(cat "$ready")"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"
export BIOMCP_CACHE_MODE=off BIOMCP_CACHE_DIR="$fixture_root/cache" BIOMCP_TEST_UNPACED_ORIGIN="$base_url"
export BIOMCP_PUBTATOR_BASE="$base_url" BIOMCP_MYVARIANT_BASE="$base_url/v1" BIOMCP_EUROPEPMC_BASE="$base_url" BIOMCP_PUBMED_BASE="$base_url/entrez/eutils" BIOMCP_S2_BASE="$base_url" BIOMCP_LITSENSE2_BASE="$base_url" BIOMCP_CLINGEN_CAR_BASE="$base_url"
panel="$repo_root/spec/fixtures/g5-v2-identity-panel.json"
all="$("$binary" --json variant articles --input "$panel" --verify-identity --debug-plan --limit 50)"
confirmed="$("$binary" --json variant articles --input "$panel" --verify-identity --confirmed-only --limit 1)"
printf 'deep-discovery\n' >"$mode"
braf_panel="$repo_root/spec/fixtures/variant-article-braf-identity-input.json"
reserved="$("$binary" --json variant articles --input "$braf_panel" --verify-identity --confirmed-only --debug-plan --limit 3)"
printf 'reordered\n' >"$mode"
reordered="$("$binary" --json variant articles --input "$panel" --verify-identity --debug-plan --limit 50)"
jq -n --argjson all "$all" --argjson confirmed "$confirmed" --argjson reserved "$reserved" --argjson reordered "$reordered" '
  def item($id): $all.items[] | select(.request_id == $id);
  def reordered_item($id): $reordered.items[] | select(.request_id == $id);
  def reserved_item: $reserved.items[] | select(.request_id == "braf-refseq-grch37");
  def has($id; $pmid; $status): any(item($id).results[]; .pmid == $pmid and .identity.status == $status);
  {
    frozen_positive_statuses: {apc: has("apc-grch38"; "12901799"; "confirmed"), atm: has("atm-grch38"; "32918381"; "confirmed"), palb2: has("palb2-grch38"; "39999518"; "confirmed"), mlh1: has("mlh1-grch38"; "20864636"; "confirmed")},
    collision_pmids_never_confirmed: (all(["31749828", "24376681", "33656647"][]; . as $pmid | any($all.items[].results[]; .pmid == $pmid and .identity.status != "confirmed"))),
    intentional_unverified: {brca1: has("brca1-grch38"; "90000003"; "unverified"), pten: has("pten-grch38"; "90000002"; "unverified"), tp53: has("tp53-grch38"; "24376681"; "unverified")},
    conflicting_observation: has("apc-grch38"; "90000001"; "conflicting"),
    outage_is_incomplete: ((item("pten-grch38").complete == false) and (item("pten-grch38").truncated == true) and (item("pten-grch38").pagination.total == null)),
    canonical_equivalence_is_additive: all(["atm-grch38", "palb2-grch38"][];
      . as $request_id | item($request_id) |
      .canonical_equivalence.caid as $caid |
      .resolution.provider_validation.status == "not_found"
      and .canonical_equivalence.status == "confirmed"
      and .canonical_equivalence.complete == true
      and .canonical_equivalence.exhaustive == true
      and ($caid | type) == "string"
      and ([.canonical_equivalence.observations[].basis] | sort) == ["genomic", "transcript_coding"]
      and all(.canonical_equivalence.observations[];
        .status == "resolved"
        and .caid == $caid
        and .comparison_complete == true
        and .source == "clingen_car"
        and (.provider_response_sha256 | test("^[0-9a-f]{64}$")))),
    confirmed_page_filters_before_limit: (any($confirmed.items[] | select(.request_id == "apc-grch38").results[]; .pmid == "12901799" and .rank == 1) and all($confirmed.items[]; .pagination.returned <= .pagination.limit and .pagination.returned == ([.results[]] | length) and all(.results[]; .identity.status == "confirmed"))),
    deep_discovery_keeps_structured_braf_for_identity_verification: (reserved_item | .complete == false and .truncated == true and .canonical_equivalence.status == "confirmed" and .canonical_equivalence.complete == true and any(.results[]; .pmid == "90000004" and .identity.status == "confirmed") and all(.results[]; .identity.status == "confirmed")),
    debug_plan_records_discovery_and_verification_allocation: (reserved_item | .debug_plan as $plan | $plan.work_allocation as $allocation | ($allocation | type) == "object" and ($allocation.discovery.limit | type) == "number" and ($allocation.discovery.consumed | type) == "number" and $allocation.discovery.consumed > 0 and $allocation.discovery.consumed <= $allocation.discovery.limit and $allocation.discovery.limit < $plan.budgets.item.limit and ($allocation.identity_verification.reserved | type) == "number" and ($allocation.identity_verification.consumed | type) == "number" and $allocation.identity_verification.reserved >= 1 and $allocation.identity_verification.consumed > 1 and $allocation.identity_verification.consumed <= $allocation.identity_verification.reserved and ($allocation.discovery.consumed + $allocation.identity_verification.consumed == $plan.budgets.item.consumed)),
    audit_versions_and_canonical_subsets: (all($all.items[]; .debug_plan.verification.verifier_version == "article-identity-v2" and (.debug_plan.verification.provider_template_version | startswith("pubtator-export")) and .debug_plan.verification.response_subset_version == "clinically-relevant-response-v1" and .debug_plan.verification.content_subset_version == "clinically-relevant-content-v1" and (.debug_plan.verification.canonical_response_subset_hash | type) == "string" and (.debug_plan.verification.canonical_content_subset_hash | type) == "string") and all($all.items[]; . as $item | reordered_item($item.request_id) | .debug_plan.verification.canonical_response_subset_hash == $item.debug_plan.verification.canonical_response_subset_hash and .debug_plan.verification.canonical_content_subset_hash == $item.debug_plan.verification.canonical_content_subset_hash)),
    typed_corresponding_gene_proof_is_pmid_bound: (all([
      ["apc-grch38", "12901799", 324, "p.Arg283Ter"],
      ["atm-grch38", "32918381", 472, "c.1066-6T>G"],
      ["palb2-grch38", "39999518", 79728, "c.3350+5G>A"]
    ][]; . as [$request_id, $pmid, $gene_id, $hgvs] |
      any(item($request_id).results[]; .pmid == $pmid and .identity.status == "confirmed" and any(.identity.observations[];
        (.provider_linkage | keys == ["expected_pmid", "gene_annotation_id", "gene_id", "identifier_tokens", "kind", "observed_hgvs", "provenance", "relation_id", "relation_roles", "relation_type", "returned_pmid", "variant_annotation_id"]) and
        .provider_linkage.kind == "pubtator_corresponding_gene" and
        .provider_linkage.expected_pmid == $pmid and .provider_linkage.returned_pmid == $pmid and
        .provider_linkage.gene_id == $gene_id and .provider_linkage.observed_hgvs == $hgvs and
        .provider_linkage.identifier_tokens == ["CorrespondingGene:\($gene_id)", "Variant:\($hgvs)"] and
        (.provider_linkage.provenance | keys == ["canonical_response_subset_sha256", "request_template_version", "response_subset_version", "source", "verifier_version"] and .source == "pubtator3" and (.canonical_response_subset_sha256 | test("^[0-9a-f]{64}$"))) and
        .provider_linkage.relation_id == null and .provider_linkage.relation_type == null and .provider_linkage.relation_roles == null and
        .gene_annotation_id == .provider_linkage.gene_annotation_id and .allele_annotation_id == .provider_linkage.variant_annotation_id and .provider_relation == null
      )))),
    document_identity_anomalies_are_incomplete_without_false_contradiction: (item("apc-grch38").results[] | select(.pmid == "12901799") | .identity.status == "confirmed" and .identity.incomplete == true and (.identity.contradictions | all(.[]; false))),
    association_without_typed_linkage_is_unverified: (item("brca1-grch38").results[] | select(.pmid == "90000003") | .identity.status == "unverified" and .identity.incomplete == true),
    expected_pmid_aggregation_is_order_independent: ((item("apc-grch38").results[] | select(.pmid == "12901799")) as $first |
      (reordered_item("apc-grch38").results[] | select(.pmid == "12901799")) as $second |
      $first.identity.status == "confirmed" and $first.identity.incomplete == true and
      $first.identity.status == $second.identity.status and $first.identity.incomplete == $second.identity.incomplete and
      ($first.identity.observations == ($first.identity.observations | unique)))
  }'
