"""Variant article rows name missing metadata only after uncovered failures.

Search scaffolding and successful recovery are synthetic. Missing-metadata
replies are unmodified upstream captures described beside the fixture bodies.
"""
from __future__ import annotations

from hang_guard import HANG_TIMEOUT

import json
import os
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

import pytest

from golden.replay_env import isolated_command_env

pytestmark = [pytest.mark.needs_binary]
ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/article/metadata-recovery"
PMIDS = ["42894377", "42891779"]


@pytest.mark.parametrize("case", ["failed", "one_missing_field", "covered", "healthy"])
def test_variant_article_rows_report_uncovered_metadata(case: str, tmp_path: Path) -> None:
    missed: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - stdlib callback
            target = urlsplit(self.path)
            params = parse_qs(target.query)
            if target.path.endswith("/esearch.fcgi"):
                return self.reply({"esearchresult": {"count": "2", "idlist": PMIDS}})
            if target.path.endswith("/efetch.fcgi"):
                return self.bytes(b"<PubmedArticleSet/>")
            if target.path.endswith("/esummary.fcgi"):
                return self.reply({"result": {"uids": PMIDS, **{
                    pmid: {"uid": pmid, "title": f"Synthetic row {pmid}",
                           "pubdate": "2026 Oct 19"} for pmid in PMIDS
                }}})
            if target.path == "/publications/export/biocjson":
                assert params["pmids"][0] in PMIDS
                if case == "healthy":
                    return self.reply({"PubTator3": [{
                        "pmid": int(params["pmids"][0]), "passages": [],
                    }]})
                if case == "one_missing_field":
                    # Authored refusal control preserves the specific provider failure class.
                    return self.reply({"error": "Synthetic refusal"}, 422)
                return self.bytes((FIXTURES / "pubtator-missing.body").read_bytes(), 400)
            if target.path == "/search" and params.get("query", [""])[0].startswith("EXT_ID:"):
                pmid = params["query"][0].split()[0].removeprefix("EXT_ID:")
                if case in ("covered", "healthy") or pmid == PMIDS[1]:
                    return self.reply({"hitCount": 1, "resultList": {"result": [{
                        "id": pmid, "pmid": pmid, "source": "MED", "title": "Synthetic recovered metadata",
                        "abstractText": f"Recovered abstract for {pmid}",
                        "citedByCount": 7,
                    }]}})
                return self.bytes((FIXTURES / f"europepmc-exact-{pmid}.body").read_bytes())
            if target.path == "/search":
                return self.reply({"hitCount": 0, "resultList": {"result": []}})
            if target.path == "/search/":
                return self.reply({"results": [], "count": 0, "total_pages": 0, "current": 1})
            if target.path.endswith(("/paper/search/bulk", "/paper/search")):
                return self.reply({"total": 0, "data": []})
            missed.append(self.path)
            self.send_error(404)

        def do_POST(self) -> None:  # noqa: N802 - stdlib callback
            size = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(size))
            self.reply([
                {"abstract": "Synthetic available abstract", "citationCount": 0}
                if case == "one_missing_field" and lookup.endswith(PMIDS[0]) else None
                for lookup in payload["ids"]
            ])

        def reply(self, body: object, status: int = 200) -> None:
            self.bytes(json.dumps(body).encode(), status)

        def bytes(self, body: bytes, status: int = 200) -> None:
            self.send_response(status)
            self.send_header("Content-Type", "application/xml" if body.startswith(b"<") else "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

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
    }, base)
    env.pop("BIOMCP_DISABLE_SOURCES", None)
    env["RUST_LOG"] = "warn"
    binary = os.environ.get("BIOMCP_BIN", str(ROOT / "target/debug/biomcp"))
    try:
        result = subprocess.run(
            [binary, "--json", "--no-cache", "variant", "articles", "Synthetic", "--strategy", "exact", "--limit", "2"],
            env=env, capture_output=True, text=True, check=False, timeout=HANG_TIMEOUT,
        )
        markdown = subprocess.run(
            [binary, "--no-cache", "variant", "articles", "Synthetic", "--strategy", "exact", "--limit", "2"],
            env=env, capture_output=True, text=True, check=False, timeout=HANG_TIMEOUT,
        )
        if case == "failed":
            request = tmp_path / "input.json"
            request.write_text(json.dumps([{"gene": "BRAF", "protein": "V600E"}]))
            batch = subprocess.run(
                [binary, "--json", "--no-cache", "variant", "articles", "--input", str(request),
                 "--strategy", "exact", "--limit", "2"],
                env=env, capture_output=True, text=True, check=False, timeout=HANG_TIMEOUT,
            )
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=HANG_TIMEOUT)
    assert result.returncode == markdown.returncode == 0, result.stderr + markdown.stderr
    assert not missed, missed
    payload = json.loads(result.stdout)
    rows = payload["results"]
    assert [row["pmid"] for row in rows] == PMIDS
    assert "metadata_status" not in rows[1], rows[1]
    if case in ("failed", "one_missing_field"):
        missing = ["abstract", "citation_count"] if case == "failed" else ["citation_count"]
        failure_class = "internal" if case == "failed" else "http_status"
        assert payload["complete"] is False
        if case == "one_missing_field":
            assert rows[0]["citation_count"] == 0
        assert rows[0].get("metadata_status") == {
            "status": "incomplete", "missing_fields": missing,
            "source": "pubtator", "failure_class": failure_class,
        }, rows[0]
        for output in (result.stderr, markdown.stderr):
            assert output.count("visible article metadata fallback") == 1, output
            assert f'class="{failure_class}"' in output, output
            assert 'pmid="42894377"' in output, output
            assert "missing_fields=" + json.dumps(missing) in output, output
        affected = next(line for line in markdown.stdout.splitlines() if line.startswith("|PMID 42894377|"))
        covered = next(line for line in markdown.stdout.splitlines() if line.startswith("|PMID 42891779|"))
        assert "incomplete" in affected and ", ".join(missing) in affected
        assert ("abstract" in affected) == (case == "failed")
        assert failure_class in affected and "pubtator" in affected
        assert "incomplete" not in covered and failure_class not in covered
        if case == "failed":
            assert batch.returncode == 0, batch.stderr + batch.stdout
            compact = json.loads(batch.stdout)["items"][0]
            assert compact["complete"] is False
            assert compact["results"][0]["metadata_status"] == rows[0]["metadata_status"]
            assert "metadata_status" not in compact["results"][1]
    else:
        assert "metadata_status" not in rows[0], rows[0]
        assert "visible article metadata fallback" not in result.stderr + markdown.stderr
        assert "Metadata status" not in markdown.stdout
        if case == "healthy":
            assert payload["complete"] is True
