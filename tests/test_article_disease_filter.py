"""Disease filters remove unrelated semantic-source rows at the CLI boundary.

The reported query and bibliographic titles reproduce captured search output.
Ontology paging uses recorded MyDisease replies; article rows and identity controls use synthetic scaffolding.
"""
from __future__ import annotations

import json
import os
import subprocess
import threading
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
TITLES = {
    "39118875": "Omission of staging PET/CT linked to reduced survival in stage III non-small cell lung cancer: insights from the LUCAS project real-world data",
    "42510157": "Pretreatment Staging FDG PET-Derived Radiomic Score Predicts Progression-Free Survival in Locally Advanced Rectal Cancer Treated with Total Neoadjuvant Therapy",
}


@contextmanager
def semantic_provider(
    tmp_path: Path, *, aliases: bool = False, lookup_failure: bool = False, melanoma: bool = False,
):
    titles = TITLES if not aliases else {
        "39118875": "Unrelated fixture condition",
        "42510157": "Fixture canonical condition: a study",
        "90000001": "Study without the label in the title",
        "90000002": "Study of prefixture aliasing",
    }
    disease_pages = []
    if melanoma:
        disease_pages = [json.loads((ROOT / "testdata/replay/responses/mydisease" / name).read_text())
                         for name in ["melanoma-page-one.json", "melanoma-page-two.json"]]
        titles = {
            "39118875": "Unrelated fixture condition",
            "42510157": "Melanoma treatment fixture",
            "90000001": "Naevocarcinoma treatment fixture",
        }
    texts = dict(titles)
    if aliases:
        texts["90000001"] = "Background context. " * 30 + "Fixture exact synonym."
    missed: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            if "/paper/search" in self.path:
                return self.reply({"total": len(titles), "data": [
                    {"paperId": pmid, "externalIds": {"PubMed": pmid}, "title": title,
                     "abstract": texts[pmid], "year": 2024, "citationCount": 1,
                     "influentialCitationCount": 0} for pmid, title in titles.items()
                ]})
            if self.path.startswith("/sentences/"):
                return self.reply([{"pmid": int(pmid), "text": text, "score": 0.9}
                                   for pmid, text in texts.items()])
            if self.path.startswith("/entrez/eutils/esummary.fcgi"):
                return self.reply({"result": {"uids": list(titles), **{
                    pmid: {"uid": pmid, "title": title, "pubdate": "2024"}
                    for pmid, title in titles.items()
                }}})
            if self.path.startswith("/query"):
                if melanoma:
                    offset = int(parse_qs(urlparse(self.path).query)["from"][0])
                    assert offset in {0, 50}, self.path
                    if lookup_failure and offset == 50:
                        return self.reply({"hits": "invalid second provider page"})
                    return self.reply(disease_pages[offset // 50])
                if lookup_failure:
                    return self.reply({"hits": "invalid provider response"})
                if aliases:
                    return self.reply({"total": 1, "hits": [{
                        "_id": "MONDO:0000001", "mondo": {
                            "name": "fixture canonical condition",
                            "synonym": ["fixture alias", "fixture exact synonym"],
                        },
                    }]})
                return self.reply({"total": 0, "hits": []})
            if self.path.startswith("/entity/autocomplete/"):
                return self.reply([])
            if self.path.startswith("/disease/"):
                if melanoma:
                    assert "MONDO:0005105" in self.path or "MONDO%3A0005105" in self.path
                    return self.reply(next(hit for hit in disease_pages[0]["hits"]
                                           if hit["_id"] == "MONDO:0005105"))
                return self.reply({"_id": "MONDO:0000001", "mondo": {
                    "name": "fixture canonical condition",
                    "synonym": ["fixture alias", "fixture exact synonym"],
                }})
            if self.path.startswith("/search/"):
                return self.reply({"results": [], "count": 0, "total_pages": 0, "current": 1})
            if self.path.startswith("/entrez/eutils/esearch.fcgi"):
                return self.reply({"esearchresult": {"count": "0", "idlist": []}})
            if self.path.startswith("/search"):
                return self.reply({"hitCount": 1, "resultList": {"result": [{
                    "id": "42494071", "pmid": "42494071", "title": "Rectal cancer fixture",
                    "abstractText": "Rectal cancer fixture", "pubYear": "2024", "citedByCount": 1,
                }]}})
            missed.append(self.path)
            self.send_error(404)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            size = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(size))
            self.reply([None for _ in payload["ids"]])

        def reply(self, body: object) -> None:
            data = json.dumps(body).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def log_message(self, format: str, *args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{server.server_port}"
    env = isolated_command_env(tmp_path, {
        "BIOMCP_PUBMED_BASE": base + "/entrez/eutils",
        "BIOMCP_PUBTATOR_BASE": base,
        "BIOMCP_EUROPEPMC_BASE": base,
        "BIOMCP_S2_BASE": base,
        "BIOMCP_LITSENSE2_BASE": base,
        "BIOMCP_MYDISEASE_BASE": base,
    }, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    try:
        yield env, titles
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
    assert not missed, missed


def search(env: dict[str, str], source: str, disease: list[str]) -> subprocess.CompletedProcess[str]:
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    return subprocess.run([
        binary, "--json", "--no-cache", "search", "article", "--source", source,
        "-k", "locally advanced initial staging routine PET CT guidelines",
        "--full", "--limit", "5", "--max-per-source", "5", *disease,
    ], env=env, capture_output=True, text=True, check=False, timeout=30)


@pytest.mark.parametrize("source", ["semanticscholar", "litsense2"])
@pytest.mark.parametrize("aliases", [False, True])
def test_disease_filter_removes_unrelated_semantic_rows(
    source: str, aliases: bool, tmp_path: Path,
) -> None:
    with semantic_provider(tmp_path, aliases=aliases) as (env, titles):
        pages = []
        term = "fixture alias" if aliases else "rectal cancer"
        for disease in [[], ["-d", term]]:
            result = search(env, source, disease)
            assert result.returncode == 0, result.stderr + result.stdout
            pages.append(json.loads(result.stdout))
    assert {row["pmid"] for row in pages[0]["results"]} == set(titles)
    expected = {"42510157", "90000001"} if aliases else {"42510157"}
    assert {row["pmid"] for row in pages[1]["results"]} == expected


def test_failed_disease_synonyms_exclude_semantic_source_and_keep_upstream_rows(tmp_path: Path) -> None:
    with semantic_provider(tmp_path, lookup_failure=True) as (env, _):
        result = search(env, "all", ["-d", "rectal cancer"])
    assert result.returncode == 0, result.stderr + result.stdout
    page = json.loads(result.stdout)
    assert page["complete"] is False
    assert [row["pmid"] for row in page["results"]] == ["42494071"]
    status = next(row for row in page["_meta"]["source_status"] if row["source"] == "semanticscholar")
    assert status["status"] in {"unavailable", "degraded"}
    assert "disease synonym lookup unavailable" in status["message"]


@pytest.mark.parametrize("source", ["semanticscholar", "litsense2"])
def test_failed_disease_synonyms_report_unavailable_semantic_source(source: str, tmp_path: Path) -> None:
    with semantic_provider(tmp_path, lookup_failure=True) as (env, _):
        result = search(env, source, ["-d", "rectal cancer"])
    assert result.returncode == 1, result.stderr + result.stdout
    page = json.loads(result.stdout)
    assert page["complete"] is False
    assert page["results"] == []
    status = next(row for row in page["_meta"]["source_status"] if row["source"] == source)
    assert status["status"] == "unavailable"
    assert "disease synonym lookup unavailable" in status["message"]


@pytest.mark.parametrize("source", ["semanticscholar", "litsense2"])
def test_melanoma_filter_uses_complete_recorded_disease_pages(source: str, tmp_path: Path) -> None:
    with semantic_provider(tmp_path, melanoma=True) as (env, _):
        result = search(env, source, ["-d", "melanoma"])
    assert result.returncode == 0, result.stderr + result.stdout
    page = json.loads(result.stdout)
    assert page["complete"] is True
    assert {row["pmid"] for row in page["results"]} == {"42510157", "90000001"}
    statuses = page["_meta"].get("source_status", [])
    assert all(row["status"] not in {"unavailable", "degraded"} for row in statuses)


@pytest.mark.parametrize("source", ["all", "semanticscholar", "litsense2"])
def test_failed_later_disease_page_preserves_other_article_sources(source: str, tmp_path: Path) -> None:
    with semantic_provider(tmp_path, melanoma=True, lookup_failure=True) as (env, _):
        result = search(env, source, ["-d", "melanoma"])
    assert result.returncode == (0 if source == "all" else 1), result.stderr + result.stdout
    page = json.loads(result.stdout)
    assert page["complete"] is False
    assert [row["pmid"] for row in page["results"]] == (["42494071"] if source == "all" else [])
    status_source = "semanticscholar" if source == "all" else source
    status = next(row for row in page["_meta"]["source_status"] if row["source"] == status_source)
    assert status["status"] == ("degraded" if source == "all" else "unavailable")
    assert "disease synonym lookup unavailable" in status["message"]
