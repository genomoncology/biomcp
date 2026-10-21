"""AUTHORED provider controls from sanitized study output; no captured replies."""

import collections
import json
import os
import pathlib
import subprocess
import sys
import threading
import time
import pytest
from hang_guard import HANG_TIMEOUT
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit, parse_qs

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env

pytestmark = [pytest.mark.needs_binary]
cases = json.loads(
    (REPO_ROOT / "tests/fixtures/article/recovery/authored.json").read_text()
)["cases"]


def execute_case(case, tmp_path, *, markdown=False):
    mode = case.get("mode", "normal")
    row_count = 25 if case.get("saturated") else 3
    req = case["requested"]
    aliases = case["resolution"]["normalized_aliases"]
    requests = []
    misses = []
    hit = {
        "_id": (aliases["genomic_ids"] or ["chr22:g.1A>G"])[0],
        "dbnsfp": {
            "genename": req["gene"],
            "hgvsp": case["source_protein_forms"],
            "hgvsc": aliases["coding_changes"],
        },
        "dbsnp": {"rsid": next(iter(aliases["rsids"]), None)},
    }
    if mode == "over-allowance":
        hit["dbnsfp"]["hgvsc"] = [
            *hit["dbnsfp"]["hgvsc"],
            *(f"c.{i:05}G>A" for i in range(250)),
        ]
    if mode == "wrong-identity":
        hit["dbnsfp"]["genename"] = "OTHER"
    hits = [hit]
    if case["resolution"]["status"] == "ambiguous":
        hits = [dict(hit, _id="chr22:g.1A>G"), dict(hit, _id="chr22:g.2A>G")]

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            t = urlsplit(self.path)
            q = parse_qs(t.query)
            requests.append((t.path, q))
            if t.path == "/query":
                body = {
                    "total": 1,
                    "hits": [
                        {
                            "symbol": req["gene"],
                            "entrezgene": int(case["gene"]["entrez_id"]),
                            "alias": case["gene"]["recorded_aliases"],
                        }
                    ],
                }
            elif t.path == "/v1/query":
                body = {"total": len(hits), "hits": hits}
            elif t.path.startswith("/v1/variant/"):
                body = hit
            elif t.path == "/allele":
                body = {"errorType": "NotFound", "description": "Authored control"}
            elif t.path == "/entity/autocomplete/" and mode == "pubtator-outage":
                self.send_error(503, "Authored PubTator outage")
                return
            elif t.path == "/entity/autocomplete/" and mode == "pubtator-empty":
                body = []
            elif t.path == "/entity/autocomplete/":
                body = [
                    {
                        "_id": f"@VARIANT_p.{req['protein_change'].removeprefix('p.')}_{req['gene']}_human",
                        "biotype": "variant",
                        "name": req["protein_change"],
                    }
                ]
            elif t.path == "/publications/export/biocjson":
                ids = q.get("pmids", [""])[0].split(",")
                body = {
                    "PubTator3": [
                        {
                            "id": i,
                            "pmid": int(i),
                            "passages": [
                                {
                                    "infons": {"type": "title"},
                                    "text": "Authored metadata control",
                                },
                                {
                                    "infons": {"type": "abstract"},
                                    "text": "Authored metadata text",
                                },
                            ],
                        }
                        for i in ids
                        if i.isdigit()
                    ]
                }
            elif t.path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                body = []
            elif t.path == "/search/":
                body = {"results": [], "count": 0, "total_pages": 0, "current": 1}
            elif t.path.endswith("/esearch.fcgi"):
                ids = (
                    [str(99001000 + i) for i in range(row_count)]
                    if case["case"] == "c003"
                    else []
                )
                term = q.get("term", [""])[0]
                short_group = (
                    mode == "grouped-pubmed-short"
                    and "p.Pro645Arg" in term
                    and " OR " in term
                )
                if short_group:
                    ids = ["99002001"]
                body = {
                    "esearchresult": {
                        "idlist": ids,
                        "count": str(len(ids) + int(short_group)),
                    }
                }
            elif t.path.endswith("/esummary.fcgi"):
                ids = q.get("id", [""])[0].split(",")
                body = {
                    "result": {
                        "uids": ids,
                        **{
                            i: {
                                "uid": i,
                                "title": "Authored route budget control",
                                "pubdate": "2026",
                                "fulljournalname": "Authored control",
                            }
                            for i in ids
                        },
                    }
                }
            elif t.path == "/search":
                term = q.get("query", [""])[0]
                metadata = "EXT_ID:" in term
                if metadata and mode == "metadata-outage":
                    self.send_error(503, "Authored metadata outage")
                    return
                rows = (
                    [
                        {
                            "id": str(99001000 + i),
                            "source": "MED",
                            "pmid": str(99001000 + i),
                            "title": "Authored route budget control",
                            "pubYear": "2026",
                        }
                        for i in range(row_count)
                    ]
                    if case["case"] == "c003"
                    else []
                )
                if case["case"] == "c003" and q.get("query", [""])[0].startswith(
                    "CHEK2 p.Gly167Arg AND NOT"
                ):
                    if case.get("saturated"):
                        rows = rows[:-1]
                    rows.append(
                        {
                            "id": "99002003",
                            "source": "MED",
                            "pmid": "99002003",
                            "title": "Authored recovered fallback control",
                            "pubYear": "2026",
                        }
                    )
                if (
                    mode in ("grouped", "grouped-europe-short")
                    and "p.Pro645Arg" in term
                    and " OR " in term
                ):
                    rows = [
                        {
                            "id": "99002001",
                            "pmid": "99002001",
                            "source": "MED",
                            "title": "Authored grouped control",
                        }
                    ]
                if metadata and mode in (
                    "unrelated-metadata",
                    "wrong-metadata-fallback",
                ):
                    ids = [
                        str(99001000 + i)
                        for i in range(row_count)
                        if str(99001000 + i) in term
                    ]
                    rows = [
                        {
                            "id": i,
                            "pmid": i,
                            "source": "MED",
                            "title": "Authored covering metadata",
                            "citedByCount": 7,
                        }
                        for i in ids
                    ]
                    for row in rows:
                        if row["id"] == "99001001" and (
                            " OR " in term or mode == "wrong-metadata-fallback"
                        ):
                            row["source"] = "PMC"
                            row["citedByCount"] = 999999
                        if row["id"] == "99001002" and (
                            " OR " in term or mode == "wrong-metadata-fallback"
                        ):
                            row["pmid"] = "99999999"
                            row["citedByCount"] = 999999
                short_group = (
                    mode == "grouped-europe-short"
                    and "p.Pro645Arg" in term
                    and " OR " in term
                )
                body = {
                    "hitCount": len(rows) + int(short_group),
                    "resultList": {"result": rows},
                }
            elif t.path in ("/graph/v1/paper/search", "/graph/v1/paper/search/bulk"):
                target = {
                    "c001": ("BRCA1 p.Pro645Arg", "99002001"),
                    "c005": ("ABRAXAS1 p.Gly93Val", "99002005"),
                }.get(case["case"])
                rows = (
                    [
                        {
                            "paperId": "authored-" + target[1],
                            "externalIds": {"PubMed": target[1]},
                            "title": "Authored recovered alias control",
                            "year": 2026,
                        }
                    ]
                    if target
                    and not mode.startswith("grouped")
                    and target[0] in q.get("query", [""])[0]
                    else []
                )
                body = {"total": len(rows), "data": rows}
            else:
                misses.append(self.path)
                self.send_error(404)
                return
            self.reply(body)

        def reply(self, body):
            b = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(b)))
            self.end_headers()
            self.wfile.write(b)

        def do_POST(self):
            d = json.loads(
                self.rfile.read(int(self.headers.get("Content-Length", "0")))
            )
            requests.append((urlsplit(self.path).path, {"ids": d.get("ids", [])}))
            self.reply([None] * len(d.get("ids", [])))

        def log_message(self, *args):
            pass

    s = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    th = threading.Thread(target=s.serve_forever, daemon=True)
    th.start()
    origin = f"http://127.0.0.1:{s.server_port}"
    providers = {
        k: origin
        for k in [
            "BIOMCP_PUBTATOR_BASE",
            "BIOMCP_EUROPEPMC_BASE",
            "BIOMCP_S2_BASE",
            "BIOMCP_LITVAR_BASE",
            "BIOMCP_CLINGEN_CAR_BASE",
            "BIOMCP_MYGENE_BASE",
            "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
        ]
    }
    providers["BIOMCP_MYVARIANT_BASE"] = origin + "/v1"
    providers["BIOMCP_PUBMED_BASE"] = origin + "/entrez/eutils"
    tmp = tmp_path / "isolated"
    env = isolated_command_env(pathlib.Path(tmp), providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    command = [
        os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp")),
        *(
            arg
            for arg in case["argv"]
            if not markdown or arg not in ("--json", "--debug-plan")
        ),
    ]
    started = time.monotonic()
    try:
        p = subprocess.run(
            command,
            env=env,
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            timeout=HANG_TIMEOUT,
        )
        elapsed = time.monotonic() - started
    finally:
        s.shutdown()
        s.server_close()
        th.join(timeout=HANG_TIMEOUT)
    assert p.returncode in (0, 1), (p.returncode, p.stdout, p.stderr)
    assert not misses, misses
    if markdown:
        return p.stdout + p.stderr, requests
    d = json.loads(p.stdout)
    assert (
        mode == "wrong-identity"
        or d["resolution"]["status"] == case["resolution"]["status"]
    ), (
        case["case"],
        d,
        p.stderr,
    )
    print(
        json.dumps(
            {
                "case": case["case"],
                "saturated": bool(case.get("saturated")),
                "mode": mode,
                "argv": case["argv"],
                "returncode": p.returncode,
                "provenance": "AUTHORED source-shaped controls; not provider replay",
                "resolution": d.get("resolution"),
                "complete": d.get("complete"),
                "truncated": d.get("truncated"),
                "results": len(d.get("results", [])),
                "elapsed_seconds": elapsed,
                "request_count": len(requests),
                "omissions": [
                    x for x in d.get("alias_coverage", []) if not x["attempted"]
                ],
                "capped_routes": [
                    x
                    for x in d.get("source_status", [])
                    if any(("cap" in y or "limit" in y for y in x["reason_codes"]))
                ],
                "budgets": d.get("debug_plan", {}).get("budgets"),
                "planned_queries": dict(
                    collections.Counter(
                        plan["route"] + ":" + plan["provider"]
                        for plan in d.get("debug_plan", {}).get("provider_queries", [])
                    )
                ),
                "requests": dict(collections.Counter((x[0] for x in requests))),
                "misses": misses,
            }
        ),
        flush=True,
    )
    return d, requests


controls = cases + [{**c, "saturated": True} for c in cases if c["case"] == "c003"]
controls += [
    {**cases[0], "mode": mode}
    for mode in (
        "grouped",
        "grouped-europe-short",
        "grouped-pubmed-short",
        "over-allowance",
        "wrong-identity",
        "pubtator-empty",
        "pubtator-outage",
    )
]
controls += [
    {**cases[1], "mode": mode}
    for mode in ("unrelated-metadata", "wrong-metadata-fallback", "metadata-outage")
]


@pytest.mark.parametrize(
    "case",
    controls,
    ids=[
        c["case"]
        + "-"
        + c.get("mode", "saturated" if c.get("saturated") else "under-cap")
        for c in controls
    ],
)
def test_capped_routes_recover_papers_and_disclose_limits(case, tmp_path):
    d, requests = execute_case(case, tmp_path)
    mode = case.get("mode", "normal")
    if mode == "wrong-identity":
        assert d["resolution"]["provider_validation"]["status"] == "contradictory"
        assert not d["results"]
        article_queries = [
            q.get("query", q.get("term", [""]))[0]
            for path, q in requests
            if path in ("/search", "/graph/v1/paper/search/bulk")
            or path.endswith("/esearch.fcgi")
        ]
        assert article_queries
        assert all(
            "BRCA1" in query and "P1749R" in query for query in article_queries
        ), article_queries
        assert all(
            "OTHER" not in query and "p.Pro645Arg" not in query
            for query in article_queries
        )
        return
    if mode == "over-allowance":
        capped = [
            r for r in d["source_status"] if "logical_work_cap" in r["reason_codes"]
        ]
        assert capped and d["complete"] is False and d["truncated"] is True
        strict_sent = {
            q.get("query", [""])[0]
            for path, q in requests
            if path == "/graph/v1/paper/search/bulk"
        }
        omitted = next(
            r
            for r in capped
            if r["route"] == "strict" and r["source"] == "semanticscholar"
        )["omitted_aliases"]
        assert omitted and all(alias not in strict_sent for alias in omitted)
        assert all(r.get("omitted_aliases") for r in capped), capped
        return
    target = {"c001": "99002001", "c003": "99002003", "c005": "99002005"}[case["case"]]
    assert target in {r["pmid"] for r in d["results"]}, (
        case["case"],
        target,
        d["source_status"],
    )
    assert not [r for r in d["alias_coverage"] if not r["attempted"]]
    ambiguous = case["resolution"]["status"] == "ambiguous"
    incomplete = (
        bool(case.get("saturated"))
        or ambiguous
        or mode
        in (
            "pubtator-empty",
            "pubtator-outage",
            "metadata-outage",
            "grouped-europe-short",
            "grouped-pubmed-short",
        )
    )
    assert d["complete"] is not incomplete
    assert d["truncated"] is incomplete
    if mode.startswith("grouped"):
        row = next(r for r in d["results"] if r["pmid"] == target)
        assert row["matched_aliases"] == []
        assert all(
            p.get("matched_alias") is None and len(p["query_aliases"]) > 1
            for p in row["provenance"]
        )
    if mode == "wrong-metadata-fallback":
        assert all(row.get("citation_count") != 999999 for row in d["results"])
    if mode == "unrelated-metadata":
        rows = [r for r in d["results"] if r["pmid"] != target]
        assert all(r["citation_count"] == 7 for r in rows)
        metadata_queries = [
            q["query"][0]
            for path, q in requests
            if path == "/search" and "EXT_ID:" in q.get("query", [""])[0]
        ]
        assert sum(" OR " in q for q in metadata_queries) == 1
        assert sum(" OR " not in q for q in metadata_queries) == 2
    if mode in ("pubtator-empty", "pubtator-outage"):
        strict = next(
            r
            for r in d["source_status"]
            if r["route"] == "strict" and r["source"] == "pubtator"
        )
        reused = next(r for r in d["source_status"] if r["route"] == "pubtator_variant")
        assert reused["status"] == strict["status"]
        assert reused["reason_codes"] == strict["reason_codes"]
        assert reused["work"]["planned"] == 0
        if mode == "pubtator-empty":
            assert sum(path == "/entity/autocomplete/" for path, _ in requests) == 1
    if mode == "metadata-outage":
        assert any(
            r["route"] == "enrichment"
            and r["source"] == "europepmc"
            and r["status"] in ("unavailable", "degraded")
            for r in d["source_status"]
        )
    if case.get("saturated") or mode in (
        "grouped-europe-short",
        "grouped-pubmed-short",
    ):
        groups = [
            r for r in d["source_status"] if "query_result_cap" in r["reason_codes"]
        ]
        assert groups and all(r["truncated_aliases"] for r in groups)
    assert not [
        r for r in d["source_status"] if "logical_work_cap" in r["reason_codes"]
    ]
    assert all((r["match_reason"] != "verified" for r in d["results"]))


def test_markdown_discloses_provider_specific_omitted_queries(tmp_path):
    text, _ = execute_case(
        {**cases[0], "mode": "over-allowance"}, tmp_path, markdown=True
    )
    assert "strict/semanticscholar: logical_work_cap. Omitted queries:" in text
    assert "source_citation/myvariant: logical_work_cap. Omitted queries:" in text
    assert "BRCA1 p.Pro645Arg" in text
    assert "chr17:g.41209100G" in text
