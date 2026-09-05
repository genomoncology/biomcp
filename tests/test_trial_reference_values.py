"""Exercise shared reference values through the real CLI and local provider replies.

Baseline replies use admitted CTGov captures. Changed replies are explicitly
synthetic derivatives for normalization and section-boundary tests.
"""

from __future__ import annotations

from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import subprocess
import threading
from urllib.parse import parse_qs, urlparse

import pytest


ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("BIOMCP_BIN", ROOT / "target/spec/biomcp"))
CAPTURES = ROOT / "testdata/sources/ctgov"


@contextmanager
def trial_server():
    replies = {
        nct_id: json.loads((CAPTURES / filename).read_text())
        for nct_id, filename in (
            ("NCT02576665", "get_nct02576665_full_20260903.json"),
            ("NCT06131398", "get_nct06131398_full_20260903.json"),
        )
    }
    requests: list[str] = []

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            requests.append(self.path)
            nct_id = urlparse(self.path).path.removeprefix("/api/v2/studies/")
            payload = replies.get(nct_id)
            body = json.dumps(payload if payload is not None else {}).encode()
            self.send_response(200 if payload is not None else 404)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *_args: object) -> None:
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}/api/v2", replies, requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def run(base: str, cache: Path, nct_id: str, section: str, *, json_output: bool = True):
    return subprocess.run(
        [BINARY, "--no-cache", *(["--json"] if json_output else []), "get", "trial", nct_id, "--source", "ctgov", section],
        cwd=ROOT,
        env=os.environ | {"BIOMCP_CTGOV_BASE": base, "BIOMCP_CACHE_DIR": str(cache)},
        text=True,
        capture_output=True,
        timeout=30,
    )


@pytest.mark.parametrize("section", ["references", "all"])
def test_recorded_references_and_empty_result_keep_section_behavior(tmp_path: Path, section: str) -> None:
    with trial_server() as (base, replies, requests):
        for nct_id in replies:
            result = run(base, tmp_path, nct_id, section)
            assert result.returncode == 0, result.stderr
            source = replies[nct_id]["protocolSection"].get("referencesModule", {}).get("references", [])
            expected = [
                {"pmid": row["pmid"], "citation": row["citation"], "reference_type": row["type"]}
                for row in source
            ]
            assert json.loads(result.stdout)["references"] == expected
            markdown = run(base, tmp_path, nct_id, section, json_output=False)
            assert markdown.returncode == 0, markdown.stderr
            assert f"https://clinicaltrials.gov/study/{nct_id}" in markdown.stdout
            positions = []
            for row in expected:
                entry = next(line for line in markdown.stdout.splitlines() if row["pmid"] in line)
                assert row["citation"] in entry
                assert row["reference_type"] in entry
                positions.append(markdown.stdout.index(row["pmid"]))
            assert positions == sorted(positions)
            if not expected:
                assert "No references" in markdown.stdout
        for request in requests:
            fields = set(parse_qs(urlparse(request).query)["fields"][0].split(","))
            assert {"ReferencePMID", "ReferenceType", "ReferenceCitation"} <= fields
            assert ("PrimaryOutcomeMeasure" in fields) == (section == "all")


def test_synthetic_partial_reply_normalizes_and_reflects_changed_references(tmp_path: Path) -> None:
    with trial_server() as (base, replies, _requests):
        protocol = replies["NCT02576665"]["protocolSection"]
        for module in ("sponsorCollaboratorsModule", "designModule", "conditionsModule"):
            protocol.pop(module, None)
        protocol["referencesModule"] = {"references": [
            {"pmid": " 12345 ", "citation": " Changed Étude α. ", "type": " PRIMARY "},
            {"pmid": None, "citation": " Citation without identifiers. ", "type": " \t "},
            {"pmid": " ", "citation": " Another citation. ", "type": None},
            {"citation": "Missing optional members."},
            {"pmid": "discard-empty", "citation": " \t "},
            {"pmid": "discard-null", "citation": None},
            {"pmid": "discard-missing"},
        ]}
        expected = [
            {"pmid": "12345", "citation": "Changed Étude α.", "reference_type": "PRIMARY"},
            {"citation": "Citation without identifiers."},
            {"citation": "Another citation."},
            {"citation": "Missing optional members."},
        ]
        result = run(base, tmp_path, "NCT02576665", "references")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["references"] == expected
        markdown = run(base, tmp_path, "NCT02576665", "references", json_output=False)
        assert markdown.returncode == 0, markdown.stderr
        assert "Changed Étude α." in markdown.stdout
        assert "Therapeutic activity" not in markdown.stdout
        assert "discard-" not in markdown.stdout


def test_other_trial_section_and_not_found_still_work(tmp_path: Path) -> None:
    with trial_server() as (base, _replies, _requests):
        result = run(base, tmp_path, "NCT02576665", "arms")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["arms"]
        missing = run(base, tmp_path, "NCT00000000", "references")
        assert missing.returncode != 0
        assert "not found" in (missing.stdout + missing.stderr).lower()
        assert "panicked" not in missing.stderr.lower()
