#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-../..}"
scenario="${2:-all}"
repo_root="$(cd "$repo_root" && pwd)"
batch_input="${3:-${repo_root}/spec/fixtures/variant-article-batch-input.json}"
fixture_root="${repo_root}/.cache/spec-variant-article-entity-${scenario}"
ready_file="${fixture_root}/ready"
server_py="${fixture_root}/server.py"
request_log="${fixture_root}/requests.log"
rm -rf "$fixture_root"
mkdir -p "$fixture_root"

cat >"$server_py" <<'PY'
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ready = Path(sys.argv[1])
request_log = Path(sys.argv[2])
scenario = sys.argv[3]

BRAF_ANNOTATION_PMID = "6010001"
BRAF_PROTEIN_ALIAS_PMID = "6010002"
BRAF_SHARED_PMID = "6010003"
BRAF_SOURCE_CITATION_PMID = "6010004"
BRAF_CODING_ALIAS_PMID = "6010005"
BRAF_GENOMIC_ALIAS_PMID = "6010006"
BRAF_SECOND_ANNOTATION_PMID = "6010007"
BRAF_PUBMED_ALIAS_PMID = "6010008"
MYD88_PMID = "24534189"
ATM_CODING_PMID = "6050001"
ATM_TRANSCRIPT_PMID = "6050002"
ATM_GENOMIC_PMID = "6050003"
PALB2_CODING_PMID = "6050004"
PALB2_TRANSCRIPT_PMID = "6050005"
PALB2_GENOMIC_PMID = "6050006"


def send_json(handler, status, payload):
    body = json.dumps(payload).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json")
    handler.send_header("Content-Length", str(len(body)))
    handler.end_headers()
    handler.wfile.write(body)


def pubtator_result(pmid, title):
    return {
        "_id": pmid,
        "pmid": pmid,
        "title": title,
        "journal": "BioMCP fixture journal",
        "date": "2024-01-01",
        "score": 42.0,
    }


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        return

    def do_GET(self):
        parsed = urlparse(self.path)
        params = parse_qs(parsed.query)
        with request_log.open("a", encoding="utf-8") as handle:
            handle.write(f"{parsed.path}?{parsed.query}\n")

        if parsed.path == "/v1/query":
            query = params.get("q", [""])[0]
            if "BRAF" in query and ("V600E" in query or "p.V600E" in query):
                send_json(self, 200, {
                    "total": 1,
                    "hits": [{
                        "_id": "chr7:g.140453136A>T",
                        "dbnsfp": {
                            "genename": ["BRAF"],
                            "hgvsp": ["p.V600E", "p.Val600Glu"],
                            "hgvsc": ["c.1799T>A"],
                        },
                    }],
                })
                return
            send_json(self, 200, {"total": 0, "hits": []})
            return

        if parsed.path.startswith("/v1/variant/"):
            if scenario == "citation-stale-json":
                send_json(self, 404, {"error": "No variant found"})
                return
            if "NC_000011.10" in parsed.path or "NC_000016.10" in parsed.path:
                send_json(self, 404, {"error": "No variant found"})
                return
            send_json(self, 200, {
                "_id": "chr7:g.140453136A>T",
                "dbnsfp": {
                    "genename": ["BRAF"],
                    "hgvsp": ["p.V600E", "p.Val600Glu"],
                    "hgvsc": ["c.1799T>A"],
                },
                "civic": {
                    "molecularProfiles": [{
                        "evidenceItems": [{
                            "source": {
                                "citation": "PMID:6010004",
                                "sourceType": "PUBMED",
                            }
                        }]
                    }]
                },
            })
            return

        if parsed.path == "/entity/autocomplete/":
            query = params.get("query", [""])[0]
            rows = []
            if query == "BRAF":
                rows.append({"_id": "@GENE_BRAF", "biotype": "Gene", "name": "BRAF"})
            if query in {"BRAF V600E", "BRAF p.V600E", "V600E", "p.V600E"}:
                rows.append({
                    "_id": "@VARIANT_p.V600V_BRAF_human",
                    "biotype": "Variant",
                    "name": "BRAF p.V600V",
                })
                rows.append({
                    "_id": "@VARIANT_p.V600E_BRAF_human",
                    "biotype": "Variant",
                    "name": "BRAF p.V600E",
                })
                rows.append({
                    "_id": "@VARIANT_p.Val600Glu_BRAF_human",
                    "biotype": "Variant",
                    "name": "BRAF p.Val600Glu",
                })
            if query == "MYD88":
                rows.append({"_id": "@GENE_MYD88", "biotype": "Gene", "name": "MYD88"})
            if query == "ATM":
                rows.append({"_id": "@GENE_ATM", "biotype": "Gene", "name": "ATM"})
            if query == "PALB2":
                rows.append({"_id": "@GENE_PALB2", "biotype": "Gene", "name": "PALB2"})
            if query in {"MYD88 S219C", "S219C", "p.S219C"}:
                rows.append({
                    "_id": "@VARIANT_p.S219C_MYD88_human",
                    "biotype": "Variant",
                    "name": "MYD88 p.S219C",
                })
            send_json(self, 200, rows)
            return

        if parsed.path == "/publications/export/biocjson":
            pmid = params.get("pmids", [""])[0]
            def passage(allele):
                return {
                    "infons": {"type": "abstract"},
                    "text": f"Captured BRAF {allele} evidence.",
                    "annotations": [
                        {"text": "BRAF", "infons": {"type": "Gene"}},
                        {"text": allele, "infons": {"type": "Mutation"}},
                    ],
                }
            passages = {
                BRAF_ANNOTATION_PMID: [passage("p.V600E")],
                BRAF_PROTEIN_ALIAS_PMID: [],
                BRAF_CODING_ALIAS_PMID: [passage("p.V600K")],
                BRAF_GENOMIC_ALIAS_PMID: [passage("p.V600E"), passage("p.V600K")],
            }.get(pmid, [])
            send_json(self, 200, {"PubTator3": [{"pmid": int(pmid), "passages": passages}]})
            return

        if parsed.path == "/search/":
            text = params.get("text", [""])[0]
            if params.get("page", ["1"])[0] != "1":
                send_json(self, 200, {
                    "results": [],
                    "count": 0,
                    "total_pages": 1,
                    "current": int(params["page"][0]),
                    "page_size": 25,
                })
                return
            if text == "@VARIANT_p.V600E_BRAF_human":
                send_json(self, 200, {
                    "results": [
                        pubtator_result(BRAF_ANNOTATION_PMID, "BRAF V600E annotation-only fixture article"),
                        pubtator_result(BRAF_SHARED_PMID, "BRAF V600E shared-route fixture article"),
                    ],
                    "count": 2,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            if text == "@VARIANT_p.Val600Glu_BRAF_human":
                send_json(self, 200, {
                    "results": [
                        pubtator_result(BRAF_SECOND_ANNOTATION_PMID, "BRAF V600E second annotation-token fixture article"),
                    ],
                    "count": 1,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            if text == "@VARIANT_p.S219C_MYD88_human":
                send_json(self, 200, {"results": [], "count": 0, "total_pages": 0, "current": 1, "page_size": 25})
                return
            if text == "MYD88 S219C":
                send_json(self, 200, {
                    "results": [pubtator_result(MYD88_PMID, "MYD88 S219C body-only free-text fixture article")],
                    "count": 1,
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            lexical_rows = {
                "BRAF V600E": [
                    pubtator_result(BRAF_SHARED_PMID, "BRAF V600E shared-route fixture article"),
                ],
                "BRAF p.V600E": [
                    pubtator_result(BRAF_SHARED_PMID, "BRAF V600E shared-route fixture article"),
                ],
                "BRAF p.Val600Glu": [
                    pubtator_result(BRAF_PROTEIN_ALIAS_PMID, "BRAF long-form protein alias fixture article"),
                ],
                "BRAF c.1799T>A": [
                    pubtator_result(BRAF_CODING_ALIAS_PMID, "BRAF coding alias fixture article"),
                ],
                "c.1799T>A": [
                    pubtator_result(BRAF_CODING_ALIAS_PMID, "BRAF coding alias fixture article"),
                ],
                "chr7:g.140453136A>T": [
                    pubtator_result(BRAF_GENOMIC_ALIAS_PMID, "BRAF genomic alias fixture article"),
                ],
                "ATM c.1066-6T>G": [
                    pubtator_result(ATM_CODING_PMID, "ATM coding alias fixture article"),
                ],
                "NM_000051.4:c.1066-6T>G": [
                    pubtator_result(ATM_TRANSCRIPT_PMID, "ATM transcript alias fixture article"),
                ],
                "NC_000011.10:g.108248927T>G": [
                    pubtator_result(ATM_GENOMIC_PMID, "ATM genomic alias fixture article"),
                ],
                "PALB2 c.3350+5G>A": [
                    pubtator_result(PALB2_CODING_PMID, "PALB2 coding alias fixture article"),
                ],
                "NM_024675.4:c.3350+5G>A": [
                    pubtator_result(PALB2_TRANSCRIPT_PMID, "PALB2 transcript alias fixture article"),
                ],
                "NC_000016.10:g.23607859C>T": [
                    pubtator_result(PALB2_GENOMIC_PMID, "PALB2 genomic alias fixture article"),
                ],
            }
            if text in lexical_rows:
                rows = lexical_rows[text]
                send_json(self, 200, {
                    "results": rows,
                    "count": len(rows),
                    "total_pages": 1,
                    "current": 1,
                    "page_size": 25,
                })
                return
            send_json(self, 200, {"results": [], "count": 0, "total_pages": 0, "current": 1, "page_size": 25})
            return

        if parsed.path.endswith("/esearch.fcgi"):
            term = params.get("term", [""])[0]
            ids = (
                [BRAF_PUBMED_ALIAS_PMID]
                if "BRAF p.Val600Glu" in term and params.get("retstart", ["0"]) == ["0"]
                else []
            )
            send_json(self, 200, {
                "esearchresult": {"idlist": ids, "count": str(len(ids))}
            })
            return

        if parsed.path.endswith("/esummary.fcgi"):
            ids = params.get("id", [])
            if ids == [BRAF_PUBMED_ALIAS_PMID]:
                send_json(self, 200, {
                    "result": {
                        "uids": ids,
                        BRAF_PUBMED_ALIAS_PMID: {
                            "uid": BRAF_PUBMED_ALIAS_PMID,
                            "title": "BRAF PubMed-only alias fixture article",
                            "sortpubdate": "2024/01/01 00:00",
                            "pubdate": "2024 Jan 1",
                            "fulljournalname": "BioMCP fixture journal",
                            "source": "BioMCP fixture journal",
                        },
                    }
                })
                return
            send_json(self, 200, {"result": {"uids": []}})
            return

        if parsed.path == "/semantic-scholar/graph/v1/paper/search" or parsed.path == "/graph/v1/paper/search":
            send_json(self, 200, {"total": 0, "data": []})
            return

        if parsed.path == "/sentences/" or parsed.path == "/passages/":
            send_json(self, 200, [])
            return

        if parsed.path == "/api/search" or parsed.path == "/search":
            send_json(self, 200, {"results": [], "hitCount": 0})
            return

        send_json(self, 200, {"resultList": {"result": []}, "hitCount": 0})


server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
ready.write_text(f"http://127.0.0.1:{server.server_port}", encoding="utf-8")
server.serve_forever()
PY

uv run --no-sync python3 "$server_py" "$ready_file" "$request_log" "$scenario" &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT

for _ in $(seq 1 100); do
  if [ -s "$ready_file" ]; then
    break
  fi
  sleep 0.05
done

base_url="$(cat "$ready_file")"
binary="${BIOMCP_BIN:-$repo_root/target/spec/biomcp}"

export BIOMCP_CACHE_MODE=off
export BIOMCP_CACHE_DIR="$fixture_root/cache"
export BIOMCP_TEST_UNPACED_ORIGIN="$base_url"
export BIOMCP_PUBTATOR_BASE="$base_url"
export BIOMCP_MYVARIANT_BASE="$base_url/v1"
export BIOMCP_EUROPEPMC_BASE="$base_url"
export BIOMCP_PUBMED_BASE="$base_url/entrez/eutils"
export BIOMCP_S2_BASE="$base_url"
export BIOMCP_LITSENSE2_BASE="$base_url"

case "$scenario" in
  all|braf)
    printf '## BRAF V600E limit 1\n'
    "$binary" variant articles "BRAF V600E" --limit 1
    printf '\n## BRAF V600E limit 3\n'
    "$binary" variant articles "BRAF V600E" --limit 3
    ;;
esac

case "$scenario" in
  all|myd88)
    printf '## MYD88 S219C fallback\n'
    "$binary" variant articles "MYD88 S219C" --limit 3
    ;;
esac

case "$scenario" in
  all|myd88-json)
    json_out="$("$binary" --json variant articles "MYD88 S219C" --limit 3)"
    jq -e '.retrieval_path | test("fallback")' >/dev/null <<<"$json_out"
    jq -e '.results | any(.pmid == "24534189")' >/dev/null <<<"$json_out"
    printf 'JSON fallback path preserved\n'
    jq -r '.results[] | select(.pmid == "24534189") | .pmid' <<<"$json_out"
    ;;
  union-json)
    full="$("$binary" --json variant articles "BRAF p.V600E" --limit 50)"
    page="$("$binary" --json variant articles "BRAF p.V600E" --limit 2 --offset 2)"
    jq -n \
      --argjson full "$full" \
      --argjson page "$page" \
      '{
        strategy: $full.strategy,
        requested_gene: $full.requested_variant.gene,
        supplied_protein: $full.requested_variant.protein_change,
        resolution: $full.resolution.status,
        complete: $full.complete,
        pmids: ([$full.results[].pmid] | sort),
        all_rows_keep_requested_variant: all($full.results[]; .requested_variant == $full.requested_variant),
        alias_matches: ([$full.results[] | select(.pmid == "6010002" or .pmid == "6010005" or .pmid == "6010006") | {pmid, matched_aliases}] | sort_by(.pmid)),
        shared_provenance: ([$full.results[] | select(.pmid == "6010003") | .provenance[]? | {route, source, matched_alias}] | sort_by(.route)),
        citation_provenance: ([$full.results[] | select(.pmid == "6010004") | .provenance[]? | {route, source, matched_alias}]),
        pubmed_provenance: ([$full.results[] | select(.pmid == "6010008") | .provenance[]? | {route, source, matched_alias}]),
        annotation_pmids: ([$full.results[] | select(any(.retrieval_routes[]?; . == "pubtator_variant")) | .pmid] | sort),
        page_matches_full_slice: ([$page.results[].pmid] == [$full.results[2:4][].pmid]),
        page_ranks: [$page.results[].rank],
        pagination: ($page.pagination | {offset, limit, returned, total, has_more}),
        truncated: $page.truncated,
        source_status: ([($full.source_status // [])[] | select((.route == "exact_lexical" and .source == "pubmed") or (.route == "pubtator_variant" and .source == "pubtator") or (.route == "source_citation" and .source == "myvariant")) | {route, source, status}] | sort_by(.route))
      }'
    ;;
  page-enrichment-json)
    page="$("$binary" --json variant articles "BRAF p.V600E" --limit 1)"
    jq -n \
      --argjson page "$page" \
      --argjson hidden_candidate_enriched "$(grep -q 'publications/export/biocjson?pmids=6010004' "$request_log" && printf true || printf false)" \
      '{
        visible_pmids: [$page.results[].pmid],
        hidden_candidate_enriched: $hidden_candidate_enriched
      }'
    ;;
  strategies-json)
    omitted="$("$binary" --json variant articles "BRAF p.V600E" --limit 10)"
    union="$("$binary" --json variant articles "BRAF p.V600E" --strategy union --limit 10)"
    annotation="$("$binary" --json variant articles "BRAF p.V600E" --strategy annotation --limit 10)"
    lexical="$("$binary" --json variant articles "BRAF p.V600E" --strategy lexical --limit 10)"
    jq -n \
      --argjson omitted "$omitted" \
      --argjson union "$union" \
      --argjson annotation "$annotation" \
      --argjson lexical "$lexical" \
      '{
        omitted_equals_union: ($omitted == $union),
        annotation_pmids: ([$annotation.results[].pmid] | sort),
        lexical_pmids: ([$lexical.results[].pmid] | sort),
        union_pmids: ([$union.results[].pmid] | sort)
      }'
    ;;
  unresolved-json)
    "$binary" --json variant articles "MYD88 S219C" --limit 3 \
      | jq '{
          resolution: .resolution.status,
          complete,
          pmid: .results[0].pmid,
          row_requested_gene: .results[0].requested_variant.gene,
          routes: .results[0].retrieval_routes,
          matched_aliases: .results[0].matched_aliases,
          has_exact_claim: ([.results[0].retrieval_routes[]?] | any(. == "pubtator_variant" or . == "exact_lexical"))
        }'
    ;;
  healthy-empty-json)
    "$binary" --json variant articles "MYD88 S219C" --strategy annotation --limit 3 \
      | jq '{
          strategy,
          resolution: .resolution.status,
          results,
          complete,
          truncated,
          pagination: (.pagination | {offset, limit, returned, total, has_more, next_page_token}),
          source_status: [(.source_status // [])[] | select(.route == "pubtator_variant" and .source == "pubtator") | {route, source, status}]
        }'
    ;;
  batch-compact-json)
    batch="$($binary --json variant articles --input "$batch_input" --limit 3)"
    followups_parseable="$({ jq -r '._meta.next_commands[]?' <<<"$batch"; } \
      | uv run --no-sync python3 -c 'import shlex, sys; rows = [line.rstrip("\n") for line in sys.stdin]; print("true" if rows and all(row and shlex.split(row) for row in rows) else "false")')"
    jq -n \
      --argjson batch "$batch" \
      --argjson followups_parseable "$followups_parseable" \
      '{
        request_ids: [$batch.items[].request_id],
        requested_genes: [$batch.items[].requested_variant.gene],
        sibling_arrays_retained: ([$batch.items[].results | type] == ["array", "array"] and all($batch.items[]; (.results | length) > 0)),
        resolutions: [$batch.items[] | {request_id, status: .resolution.status}],
        match_reasons: {
          braf_all_exact: (all($batch.items[] | select(.request_id == "braf-v600e") | .results[]; .match_reason == "exact_variant")),
          myd88_all_best_effort: (all($batch.items[] | select(.request_id == "myd88-s219c") | .results[]; .match_reason == "best_effort_free_text"))
        },
        route_claims: {
          braf_has_exact: (any($batch.items[] | select(.request_id == "braf-v600e") | .results[].routes[]; . == "pubtator_variant" or . == "exact_lexical" or . == "source_citation")),
          myd88_only_fallback: (
            any($batch.items[] | select(.request_id == "myd88-s219c") | .results[].routes[]; . == "best_effort_free_text")
            and all($batch.items[] | select(.request_id == "myd88-s219c") | .results[].routes[]; . == "best_effort_free_text"))
        },
        aggregate: {complete: $batch.complete, truncated: $batch.truncated},
        item_state_present: (all($batch.items[]; (.complete | type) == "boolean" and (.truncated | type) == "boolean" and (.pagination | type) == "object" and (.source_status | type) == "array" and has("error"))),
        compact_rows: (all($batch.items[].results[];
          ((.pmid // .pmcid // .doi // .arxiv_id // .semantic_scholar_id) | type) == "string"
          and (.title | type) == "string"
          and (.date | type) == "string"
          and (.matched_aliases | type) == "array"
          and (.routes | type) == "array"
          and (.sources | type) == "array"
          and (.rank | type) == "number"
          and has("is_retracted")
          and (has("abstract") or has("abstract_snippet") or has("full_text") or has("annotations") or has("provenance") or has("ranking") | not))),
        followups: {
          parseable: $followups_parseable,
          article_batch: (any($batch._meta.next_commands[]; startswith("biomcp article batch "))),
          article_detail: (any($batch._meta.next_commands[]; startswith("biomcp get article ") and (split(" ") | length) == 4)),
          fulltext: (any($batch._meta.next_commands[]; startswith("biomcp get article ") and endswith(" fulltext"))),
          assets: (any($batch._meta.next_commands[]; startswith("biomcp get article ") and endswith(" assets"))),
          citations: (any($batch._meta.next_commands[]; startswith("biomcp article citations ")))
        }
      }'
    ;;
  citation-stale-json)
    "$binary" --json variant articles "BRAF p.V600E" --limit 3 \
      | jq '{
          resolution: .resolution.status,
          source_citation: ([.source_status[] | select(.route == "source_citation" and .source == "myvariant") | {status}][0]),
          complete,
          truncated
        }'
    ;;
  refseq-not-found-json)
    refseq_input="${repo_root}/spec/fixtures/variant-article-refseq-input.json"
    batch="$($binary --json variant articles --input "$refseq_input" --limit 10 --debug-plan)"
    jq -n \
      --argjson batch "$batch" \
      'def expected_aliases:
         if .requested_variant.gene == "ATM" then
           ["ATM c.1066-6T>G", "NC_000011.10:g.108248927T>G", "NM_000051.4:c.1066-6T>G"]
         else
           ["NC_000016.10:g.23607859C>T", "NM_024675.4:c.3350+5G>A", "PALB2 c.3350+5G>A"]
         end;
       def route_queries: [.debug_plan.routes[] | {route, queries}] | sort_by(.route);
       def source_citation_status:
         [.source_status[] | select(.route == "source_citation" and .source == "myvariant") | {status, detail}][0];
       def exact_result_shape:
         [.results[] | {matched_aliases, routes, sources}] | sort_by(.matched_aliases, .routes, .sources);
       ($batch.items[] | select(.request_id == "atm-grch38")) as $atm_components
       | ($batch.items[] | select(.request_id == "atm-grch38-genomic")) as $atm_genomic
       | {
           items: [
             $batch.items[]
             | select(.request_id == "atm-grch38" or .request_id == "palb2-grch38")
             | . as $item
             | ($item | expected_aliases) as $expected
             | {
                 request_id,
                 requested_variant,
                 resolution,
                 complete,
                 truncated,
                 source_citation: source_citation_status,
                 literal_exact_aliases: ([.results[].matched_aliases[]] | unique),
                 only_literal_exact_aliases: (([.results[].matched_aliases[]] | unique) == $expected),
                 literal_exact_route_queries: ([.debug_plan.routes[] | select(.route == "exact_lexical") | .queries[]] | unique),
                 only_literal_route_queries: (([.debug_plan.routes[].queries[]] | unique) - $expected | length == 0),
                 literal_route_source_provenance: all($expected[];
                   . as $alias
                   | any($item.results[];
                     (.matched_aliases == [$alias])
                     and (.routes == ["exact_lexical"])
                     and (.sources == ["pubtator"])))
               }
           ],
           encoding_equivalence: {
             same_requested_variant: ($atm_components.requested_variant == $atm_genomic.requested_variant),
             expected_normalized_aliases: ($atm_components.resolution.normalized_aliases == {
               protein_changes: [],
               coding_changes: ["c.1066-6T>G"],
               genomic_ids: ["NC_000011.10:g.108248927T>G"],
               rsids: []
             }),
             same_normalized_aliases: ($atm_components.resolution.normalized_aliases == $atm_genomic.resolution.normalized_aliases),
             same_route_queries: (($atm_components | route_queries) == ($atm_genomic | route_queries)),
             same_public_behavior: (
               $atm_components.resolution == $atm_genomic.resolution
               and $atm_components.complete == $atm_genomic.complete
               and $atm_components.truncated == $atm_genomic.truncated
               and ($atm_components | source_citation_status) == ($atm_genomic | source_citation_status)
               and ($atm_components | exact_result_shape) == ($atm_genomic | exact_result_shape)
             )
           }
         }'
    ;;
  identity-verification-json)
    verified="$($binary --json variant articles "BRAF p.V600E" --verify-identity --debug-plan --limit 10)"
    confirmed="$($binary --json variant articles "BRAF p.V600E" --verify-identity --confirmed-only --limit 1)"
    jq -n \
      --argjson verified "$verified" \
      --argjson confirmed "$confirmed" \
      '{
        normal_statuses: ([$verified.results[] | select(.pmid == "6010001" or .pmid == "6010002" or .pmid == "6010005" or .pmid == "6010006") | {pmid, status: .identity.status}] | sort_by(.pmid)),
        alias_only_candidates_never_confirmed: (all($verified.results[] | select(.pmid == "6010002" or .pmid == "6010005" or .pmid == "6010006"); .identity.status != "confirmed")),
        confirmed_observation_is_auditable: ([$verified.results[] | select(.identity.status == "confirmed") | .identity.observations[] | {
          source: ((.source | type) == "string" and (.source | length) > 0),
          section: ((.section | type) == "string" and (.section | length) > 0),
          locator: ((.locator | type) == "string" and (.locator | length) > 0),
          linked_gene: (.linked_gene == "BRAF"),
          observed_alias: ((.observed_alias | type) == "string" and (.observed_alias | length) > 0),
          canonical_content_hash: ((.canonical_content_hash | type) == "string" and (.canonical_content_hash | length) > 0)
        }] | length > 0 and all(.[]; .source and .section and .locator and .linked_gene and .observed_alias and .canonical_content_hash)),
        confirmed_only_keeps_the_confirmed_result: any(
          $confirmed.results[];
          .pmid == "6010001" and .identity.status == "confirmed" and .rank == 1
        ),
        confirmed_only_excludes_nonconfirmations: all(
          $confirmed.results[];
          .identity.status == "confirmed"
        ),
        debug_plan_records_verification_artifact: (
          ($verified.debug_plan.verification.verifier_version | type) == "string"
          and ($verified.debug_plan.verification.provider_template_version | type) == "string"
          and ($verified.debug_plan.verification.artifact_id | type) == "string"
          and ($verified.debug_plan.verification.response_hashes_are_post_response == true)
          and ($verified.debug_plan.verification.captured_content_hashes_are_post_response == true)
        )
      }'
    ;;
  debug-plan-json)
    ordinary_single="$($binary --json variant articles "BRAF p.V600E" --limit 3)"
    ordinary_batch="$($binary --json variant articles --input "$batch_input" --limit 3)"
    single="$($binary --json variant articles "BRAF p.V600E" --limit 3 --debug-plan)"
    batch="$($binary --json variant articles --input "$batch_input" --limit 3 --debug-plan)"
    jq -n \
      --argjson ordinary_single "$ordinary_single" \
      --argjson ordinary_batch "$ordinary_batch" \
      --argjson single "$single" \
      --argjson batch "$batch" \
      'def provider_facts:
         length > 0 and all(.[];
           (.source | type) == "string"
           and (.status | IN("ok", "degraded", "unavailable", "skipped"))
           and (.latency_ms | type) == "number" and .latency_ms >= 0
           and (.calls | type) == "number" and .calls >= 0
           and (.pages | type) == "number" and .pages >= 0
           and (.cache | IN("hit", "miss", "bypass", "mixed", "unavailable", "not_applicable")));
       def budget_consistent:
         (.limit | type) == "number"
         and (.consumed | type) == "number"
         and (.remaining | type) == "number"
         and (.exhausted | type) == "boolean"
         and .consumed + .remaining == .limit;
       def item_plan_shape:
         (.normalized_aliases | type) == "object"
         and ([.routes[].queries[]?] | length) > 0
         and ([.routes[].providers[]] | provider_facts)
         and (.counts.pre_dedup | type) == "number"
         and (.counts.post_dedup | type) == "number"
         and (.counts.returned | type) == "number"
         and ([.ranking.inputs[]] | index("exactness") != null)
         and ([.ranking.inputs[]] | index("route_source_position") != null)
         and ([.ranking.inputs[]] | index("stable_identifier") != null)
         and (.budgets.item | budget_consistent)
         and (.budgets.request | budget_consistent)
         and (.truncated | type) == "boolean"
         and (.stopped_routes | type) == "array"
         and (.next.offset | type) == "number"
         and (.next | has("cursor"));
       {
         ordinary_omits_plan: {
           single: ($ordinary_single | has("debug_plan") | not),
           batch: ($ordinary_batch | has("debug_plan") | not)
         },
         single: {
           aliases_present: (($single.debug_plan.normalized_aliases | to_entries | map(.value | length) | add) > 0),
           required_routes: {
             annotation: ([$single.debug_plan.routes[].route] | index("pubtator_variant") != null),
             lexical: ([$single.debug_plan.routes[].route] | index("exact_lexical") != null),
             source_citation: ([$single.debug_plan.routes[].route] | index("source_citation") != null)
           },
           shape_complete: ($single.debug_plan | item_plan_shape)
         },
         batch: {
           item_concurrency_limit: $batch.debug_plan.item_concurrency_limit,
           items_planned: $batch.debug_plan.items_planned,
           request_budget_consistent: ($batch.debug_plan.work | budget_consistent),
           every_item_has_plan: (
             ($batch.items | length) > 0
             and all($batch.items[]; has("debug_plan") and (.debug_plan | item_plan_shape)))
         }
       }'
    ;;
esac
