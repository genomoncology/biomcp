from __future__ import annotations

import os
import json
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit


REPO_ROOT = Path(__file__).resolve().parents[1]


class CountingHandler(BaseHTTPRequestHandler):
    requests = 0

    def do_GET(self) -> None:  # noqa: N802
        type(self).requests += 1
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"data":[],"total":0}')

    def log_message(self, *_args: object) -> None:
        pass


def test_rejected_nci_filters_never_reach_local_transport() -> None:
    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
    assert binary.exists(), f"missing biomcp binary: {binary}"
    server = ThreadingHTTPServer(("127.0.0.1", 0), CountingHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    env = os.environ | {
        "NCI_API_KEY": "test-key",
        "BIOMCP_NCI_CTS_BASE": f"http://127.0.0.1:{server.server_port}",
    }
    rejected = [
        ["--study-type", "interventional"],
        ["--sponsor", "NCI"],
        ["--date-from", "2026-01-01"],
        ["--date-to", "2026-01-01"],
        ["--biomarker", "BRAF", "--mutation", "V600E"],
        ["--biomarker", "BRAF", "V600E"],
    ]
    try:
        for filters in rejected:
            result = subprocess.run(
                [binary, "search", "trial", "--source", "nci", *filters],
                cwd=REPO_ROOT,
                env=env,
                text=True,
                capture_output=True,
                check=False,
            )
            assert result.returncode != 0, (filters, result.stdout, result.stderr)
        assert CountingHandler.requests == 0
    finally:
        server.shutdown()
        thread.join()
        server.server_close()


def test_nci_detail_executes_the_biodata_plan_through_the_real_cli() -> None:
    response = (
        REPO_ROOT / "testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json"
    ).read_bytes()
    fields = [
        "nci_id",
        "nct_id",
        "brief_title",
        "official_title",
        "current_trial_status",
        "why_study_stopped",
        "study_protocol_type",
        "phase",
        "diseases",
        "minimum_target_accrual_number",
        "arms",
        "lead_org",
        "start_date",
        "completion_date",
        "eligibility",
        "brief_summary",
    ]
    class DetailHandler(BaseHTTPRequestHandler):
        request_path = ""
        query: list[tuple[str, str]] = []
        api_keys: list[str] = []

        def do_GET(self) -> None:  # noqa: N802
            parsed = urlsplit(self.path)
            type(self).request_path = parsed.path
            type(self).query = parse_qsl(parsed.query, keep_blank_values=True)
            type(self).api_keys = self.headers.get_all("X-API-KEY", failobj=[])
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(response)

        def log_message(self, *_args: object) -> None:
            pass

    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
    assert binary.exists(), f"missing biomcp binary: {binary}"
    server = ThreadingHTTPServer(("127.0.0.1", 0), DetailHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    env = os.environ | {
        "NCI_API_KEY": "detail-secret",
        "BIOMCP_NCI_CTS_BASE": f"http://127.0.0.1:{server.server_port}",
    }
    try:
        result = subprocess.run(
            [binary, "--json", "get", "trial", "NCT05879926", "--source", "nci", "all"],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        markdown = subprocess.run(
            [binary, "get", "trial", "NCT05879926", "--source", "nci", "arms"],
            cwd=REPO_ROOT,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
    finally:
        server.shutdown()
        thread.join()
        server.server_close()

    assert result.returncode == 0, result.stderr
    assert markdown.returncode == 0, markdown.stderr
    assert "and 20 more" in markdown.stdout
    assert "and 23 more" in markdown.stdout
    trial = json.loads(result.stdout)
    assert len(trial["arms"]) == 2
    assert len(trial["interventions"]) == 53
    assert len(trial["arm_intervention_assignments"]) == 53
    assert len({row["id"] for row in trial["interventions"]}) == 53
    assert "intervention_details" not in trial
    assert DetailHandler.request_path == "/trials"
    assert DetailHandler.query == [
        ("size", "1"),
        ("nct_id", "NCT05879926"),
        *[("include", field) for field in fields],
    ]
    assert DetailHandler.api_keys == ["detail-secret"]
    assert "detail-secret" not in result.stdout + result.stderr
    trial = json.loads(result.stdout)
    assert trial["nct_id"] == "NCT05879926"
    assert trial["source"] == "NCI CTS"
    assert trial["age_range"] == "18 Years to Any age"
    assert trial["eligibility"]["maximum_age"]["original"] == "999 Years"
    assert trial["eligibility_text"].startswith("Inclusion Criteria:\n- ")
