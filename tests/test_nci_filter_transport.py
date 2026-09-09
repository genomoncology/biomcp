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


def test_nci_detail_executes_the_strict_biodata_plan_through_the_real_cli() -> None:
    receipted_response = json.loads(
        (
            REPO_ROOT / "testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json"
        ).read_text(encoding="utf-8")
    )
    # The receipt is a broad search capture. The detail plan is an exact filtered
    # size-one request, so its response envelope reports the one matching row.
    receipted_response["total"] = 1
    response = json.dumps(receipted_response, separators=(",", ":")).encode()
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
        request_paths: list[str] = []
        queries: list[list[tuple[str, str]]] = []
        api_keys: list[str] = []

        def do_GET(self) -> None:  # noqa: N802
            parsed = urlsplit(self.path)
            type(self).request_paths.append(parsed.path)
            type(self).queries.append(parse_qsl(parsed.query, keep_blank_values=True))
            type(self).api_keys.extend(self.headers.get_all("X-API-KEY", failobj=[]))
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
        "BIOMCP_TEST_UNPACED_ORIGIN": f"http://127.0.0.1:{server.server_port}",
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
            [
                binary,
                "get",
                "trial",
                "NCT05879926",
                "--source",
                "nci",
                "eligibility",
                "arms",
            ],
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
    assert "Sex: Female" in markdown.stdout
    assert "Eligible Ages: 18 Years to Any age" in markdown.stdout
    assert "Healthy Subjects: No" in markdown.stdout
    criteria = json.loads(response)["data"][0]["eligibility"]["unstructured"]
    normalized_markdown = "\n".join(
        line.rstrip() for line in markdown.stdout.splitlines()
    )
    prior = -1
    for row in sorted(criteria, key=lambda item: item["display_order"]):
        description = "\n".join(
            line.rstrip() for line in row["description"].splitlines()
        )
        position = normalized_markdown.find(description)
        assert position > prior, row["display_order"]
        prior = position
    inclusion = markdown.stdout.index("### Inclusion Criteria")
    exclusion = markdown.stdout.index("### Exclusion Criteria")
    assert inclusion < exclusion
    assert markdown.stdout.count("### Inclusion Criteria") == 1
    assert markdown.stdout.count("### Exclusion Criteria") == 1
    trial = json.loads(result.stdout)
    assert len(trial["arms"]) == 2
    assert len(trial["interventions"]) == 53
    assert len(trial["arm_intervention_assignments"]) == 53
    assert len({row["id"] for row in trial["interventions"]}) == 53
    assert "intervention_details" not in trial
    assert DetailHandler.request_paths == ["/trials", "/trials"]
    assert DetailHandler.queries[0] == [
        ("size", "1"),
        ("nct_id", "NCT05879926"),
        *[("include", field) for field in fields],
    ]
    assert DetailHandler.queries[1] == [
        ("size", "1"),
        ("nct_id", "NCT05879926"),
        *[("include", field) for field in fields],
    ]
    assert DetailHandler.api_keys == ["detail-secret", "detail-secret"]
    assert "detail-secret" not in result.stdout + result.stderr
    trial = json.loads(result.stdout)
    assert trial["nct_id"] == "NCT05879926"
    assert trial["source"] == "NCI CTS"
    eligibility = trial["eligibility"]
    assert eligibility["age_range"]["minimum"]["source"] == "18 Years"
    assert eligibility["age_range"]["maximum"]["kind"] == "source_stated_no_limit"
    assert eligibility["age_range"]["maximum"]["source"] == "999 Years"
    assert eligibility["sexes"][0]["authority"] == "nci"
    assert eligibility["sexes"][0]["code"] == "FEMALE"
    assert eligibility["includes_healthy_subjects"] is False
    assert len(eligibility["criteria"]) == 36
    assert [row["id"] for row in eligibility["criteria"][:3]] == [1, 2, 3]
    assert "age_range" not in trial
    assert "eligibility_text" not in trial
