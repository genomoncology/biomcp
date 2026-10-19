"""Literal paper transcript anchors require authoritative allele equivalence."""

from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent / "golden"))
from replay_env import REPO_ROOT, isolated_command_env  # noqa: E402

pytestmark = [pytest.mark.needs_binary]
FIXTURES = REPO_ROOT / "tests/fixtures/article/paper-anchors"
PAPER = REPO_ROOT / (
    "tests/fixtures/article/transcript-aliases/replay/responses/pubtator/"
    "pubtator-publications-export-biocjson-379b0ee3c1d5.bin"
)


@pytest.mark.parametrize("case", [
    "same_allele", "different_allele", "unresolved_anchor", "missing_typed_gene",
    "wrong_variant_span", "unversioned_anchor", "wrong_pmid", "split_document_linkage", "oversized_anchor",
])
def test_literal_paper_transcript_anchor_requires_resolved_same_allele(
    case: str, tmp_path: Path,
) -> None:
    # Discovery is deliberately synthetic. Paper text and CAR responses are recorded.
    paper = json.loads(PAPER.read_bytes())
    document = paper["PubTator3"][0]
    assert str(document["pmid"]) == "32272925"
    assert "NM_007294.3:c.3748G > T" in document["passages"][43]["text"]
    # These mutations are explicit refusal controls, not captured provider facts.
    if case == "split_document_linkage":
        unanchored = copy.deepcopy(document)
        unanchored["passages"][43]["text"] = unanchored["passages"][43]["text"].replace(
            "NM_007294.3:", "NM_007294  :",
        )
        paper["PubTator3"].append(unanchored)
    if case in ("missing_typed_gene", "split_document_linkage"):
        for passage in document["passages"]:
            passage["annotations"] = [annotation for annotation in passage["annotations"]
                                      if annotation.get("infons", {}).get("type") != "Gene"]
    elif case == "wrong_variant_span":
        for passage in document["passages"]:
            for annotation in passage["annotations"]:
                for location in annotation.get("locations", []):
                    location["offset"] += 1
    elif case == "unversioned_anchor":
        document["passages"][43]["text"] = document["passages"][43]["text"].replace(
            "NM_007294.3:", "NM_007294  :",
        )
    elif case == "wrong_pmid":
        document["pmid"] = 99999999
    elif case == "oversized_anchor":
        oversized = "c.3748G" + " " * 4096 + "> T"
        paragraph = document["passages"][43]
        paragraph["text"] = paragraph["text"].replace("c.3748G > T", oversized)
        for annotation in paragraph["annotations"]:
            if annotation["id"] == "21841":
                annotation["locations"][0]["length"] = len(oversized)
    coding = "c.3607G>C" if case == "different_allele" else "c.3607G>T"
    requested_query = f"NM_007297.4:{coding}"
    car = {
        "NM_007294.3:c.3748G>T": (FIXTURES / "car-nm_007294.3-c.3748g-t-projected.json").read_bytes(),
        "NM_007297.4:c.3607G>T": (FIXTURES / "car-nm_007297.4-c.3607g-t-projected.json").read_bytes(),
        "NM_007297.4:c.3607G>C": (FIXTURES / "car-nm_007297.4-c.3607g-c-projected.json").read_bytes(),
    }
    paper_caid = json.loads(car["NM_007294.3:c.3748G>T"])["@id"]
    requested_caid = json.loads(car[requested_query])["@id"]
    assert (paper_caid == requested_caid) == (case != "different_allele")
    requests: list[tuple[str, dict[str, list[str]]]] = []
    misses: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def reply(self, value: object, status: int = 200) -> None:
            body = value if isinstance(value, bytes) else json.dumps(value).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self) -> None:  # noqa: N802
            target = urlsplit(self.path)
            query = parse_qs(target.query)
            path = target.path
            requests.append((path, query))
            if path == "/allele":
                hgvs = query.get("hgvs", [""])[0]
                if case == "unresolved_anchor" and hgvs == "NM_007294.3:c.3748G>T":
                    return self.reply({"error": "Simulated unavailable paper anchor"}, 503)
                if hgvs in car:
                    return self.reply(car[hgvs])
            elif path == "/v1/query":
                return self.reply({"total": 0, "hits": []})
            elif path.startswith("/v1/variant/"):
                return self.reply({"error": "not found"}, 404)
            elif path in ("/entity/autocomplete/", "/variant/autocomplete/"):
                return self.reply([])
            elif path == "/search/":
                return self.reply({"results": [{
                    "_id": "32272925", "pmid": 32272925, "pmcid": "PMC7146901",
                    "title": document["passages"][0]["text"], "score": 1,
                }], "count": 1, "total_pages": 1, "current": 1, "page_size": 25})
            elif path == "/publications/export/biocjson":
                return self.reply(paper)
            elif path.endswith("/esearch.fcgi"):
                return self.reply({"esearchresult": {"idlist": [], "count": "0"}})
            elif path == "/search":
                return self.reply({"hitCount": 0, "resultList": {"result": []}})
            elif path in ("/graph/v1/paper/search", "/graph/v1/paper/search/bulk"):
                return self.reply({"total": 0, "data": []})
            elif path.startswith("/ldh/Variant/id/"):
                return self.reply({"status": {"code": 200}, "metadata": {}, "data": {"VariantsInLiterature": []}})
            misses.append(self.path)
            self.reply({"error": "No recorded response"}, 404)

        def do_POST(self) -> None:  # noqa: N802
            self.rfile.read(int(self.headers.get("Content-Length", "0")))
            if urlsplit(self.path).path == "/graph/v1/paper/batch":
                return self.reply([None])
            misses.append(self.path)
            self.reply({"error": "No recorded response"}, 404)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    origin = f"http://127.0.0.1:{server.server_port}"
    providers = {key: origin for key in (
        "BIOMCP_PUBTATOR_BASE", "BIOMCP_EUROPEPMC_BASE", "BIOMCP_S2_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE",
        "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    )}
    providers["BIOMCP_MYVARIANT_BASE"] = origin + "/v1"
    providers["BIOMCP_PUBMED_BASE"] = origin + "/entrez/eutils"
    env = isolated_command_env(tmp_path / "isolated", providers, origin)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    input_path = tmp_path / "input.json"
    input_path.write_text(json.dumps([{
        "gene": "BRCA1", "transcript": "NM_007297.4", "coding": coding,
    }]))
    binary = os.environ.get("BIOMCP_BIN", str(REPO_ROOT / "target/debug/biomcp"))
    command = [binary, "--json", "--no-cache", "variant", "articles", "--input",
               str(input_path), "--verify-identity", "--debug-plan", "--limit", "5"]
    try:
        result = subprocess.run(command, env=env, cwd=REPO_ROOT,
                                capture_output=True, text=True, timeout=60)
        assert result.returncode == 0, result.stdout + result.stderr
        payload = json.loads(result.stdout)["items"][0]
        assert payload["canonical_equivalence"]["caid"] == requested_caid.rsplit("/", 1)[-1]
        assert payload["canonical_equivalence"]["exhaustive"] is True
        assert payload["canonical_equivalence"]["observations"][0]["status"] == "resolved"
        row = next(row for row in payload["results"] if row["pmid"] == "32272925")
        print(json.dumps({"case": case, "requested_query": requested_query,
                          "requested_caid": requested_caid, "paper_caid": paper_caid,
                          "identity": row["identity"], "car_requests": [query for path, query in requests if path == "/allele"]}))
        assert not misses, misses
        assert row["identity"]["status"] == (
            "confirmed" if case == "same_allele" else "unverified"
        )
        paper_queries = [query["hgvs"][0] for path, query in requests if path == "/allele"]
        anchor_query = "NM_007294.3:c.3748G>T"
        if case in ("same_allele", "different_allele"):
            assert paper_queries.count(anchor_query) == 1
        elif case == "unresolved_anchor":
            # The shared provider client retries HTTP failures within one logical unit.
            assert anchor_query in paper_queries
        else:
            assert anchor_query not in paper_queries
        if case == "same_allele":
            observation = row["identity"]["observations"][0]
            linkage = observation["provider_linkage"]
            assert linkage["kind"] == "pubtator_transcript_anchor"
            assert linkage["caid"] == "CA002402"
            assert linkage["literal_anchor"] == "NM_007294.3:c.3748G > T"
            assert linkage["resolution"]["provider_response_sha256"] == (
                "e497b293de0dd11574a38d6652a8f3568dfbc984234709da59595ad717e44d85"
            )
            assert linkage["requested_resolutions"][0]["query"] == requested_query
            assert linkage["paper_linkage"]["variant_annotation_id"] == "21841"
            assert ":variant-passage:44:offset:13292:length:11" in observation["locator"]
        if case == "unresolved_anchor":
            assert row["identity"]["incomplete"] is True
            source = next(status for status in payload["source_status"]
                          if status["route"] == "identity_verification"
                          and status["source"] == "clingen_car")
            assert source["status"] == "unavailable"
            assert source["work"]["planned"] == 1
            assert source["work"]["unavailable"] == 1
        if case != "same_allele":
            confirmed = subprocess.run(command + ["--confirmed-only"], env=env,
                                       cwd=REPO_ROOT, capture_output=True, text=True,
                                       timeout=60)
            assert confirmed.returncode == (1 if case == "unresolved_anchor" else 0), (
                confirmed.stdout + confirmed.stderr
            )
            filtered = json.loads(confirmed.stdout)["items"][0]
            assert filtered["results"] == []
            if case == "unresolved_anchor":
                assert filtered["error"]["code"] == "source_unavailable"
            assert not misses, misses
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
