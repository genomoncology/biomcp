from __future__ import annotations

import os
import json
import subprocess
import threading
from collections import Counter
from copy import deepcopy
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qsl, urlsplit


REPO_ROOT = Path(__file__).resolve().parents[1]


def _run_nci_detail(
    response: bytes, *arguments: str
) -> subprocess.CompletedProcess[str]:
    class DetailHandler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
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
        "NCI_API_KEY": "case21-credential-sentinel",
        "BIOMCP_NCI_CTS_BASE": f"http://127.0.0.1:{server.server_port}",
        "BIOMCP_TEST_UNPACED_ORIGIN": f"http://127.0.0.1:{server.server_port}",
    }
    try:
        return subprocess.run(
            [binary, *arguments],
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


def _case21_response() -> dict[str, object]:
    response = json.loads(
        (
            REPO_ROOT / "testdata/sources/nci_cts/get_nci_2023_04529_full_20260903.json"
        ).read_text(encoding="utf-8")
    )
    response["total"] = 1
    return response


def _case21_rows(response: dict[str, object]) -> list[tuple[int, dict[str, object]]]:
    rows = response["data"][0]["eligibility"]["unstructured"]  # type: ignore[index]
    return sorted(enumerate(rows), key=lambda item: item[1]["display_order"])


def _case21_command(response: dict[str, object], json_output: bool) -> tuple[str, str]:
    arguments = ["get", "trial", "NCT05879926", "--source", "nci", "eligibility"]
    if json_output:
        arguments.insert(0, "--json")
    result = _run_nci_detail(json.dumps(response).encode(), *arguments)
    assert result.returncode == 0, result.stderr
    return result.stdout, result.stderr


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


def test_nci_search_page_preserves_public_result_and_private_credential() -> None:
    response = (
        REPO_ROOT / "testdata/sources/nci_cts/search_melanoma_20260811.json"
    ).read_bytes()

    class SearchHandler(BaseHTTPRequestHandler):
        paths: list[str] = []
        api_keys: list[str] = []

        def do_GET(self) -> None:  # noqa: N802
            type(self).paths.append(self.path)
            type(self).api_keys.extend(self.headers.get_all("X-API-KEY", failobj=[]))
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(response)))
            self.end_headers()
            self.wfile.write(response)

        def log_message(self, *_args: object) -> None:
            pass

    binary = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/debug/biomcp"))
    server = ThreadingHTTPServer(("127.0.0.1", 0), SearchHandler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    credential = "nci-search-credential-sentinel-0117"
    env = os.environ | {
        "NCI_API_KEY": credential,
        "BIOMCP_NCI_CTS_BASE": f"http://127.0.0.1:{server.server_port}",
        "BIOMCP_TEST_UNPACED_ORIGIN": f"http://127.0.0.1:{server.server_port}",
    }
    try:
        result = subprocess.run(
            [
                binary,
                "--json",
                "search",
                "trial",
                "--source",
                "nci",
                "--phase",
                "3",
                "--limit",
                "1",
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
    payload = json.loads(result.stdout)
    assert payload["pagination"]["total"] == 2112
    assert payload["results"] == [
        {
            "nct_id": "NCT05929768",
            "title": "Shorter Chemo-Immunotherapy Without Anthracycline Drugs for Early Triple Negative Breast Cancer",
            "status": "Active",
            "phase": "III",
            "conditions": [row["name"] for row in json.loads(response)["data"][0]["diseases"]],
            "sponsor": "SWOG",
        }
    ]
    assert len(SearchHandler.paths) == 1
    assert SearchHandler.api_keys == [credential]
    assert credential not in result.stdout + result.stderr


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
    source_trial = json.loads(response)["data"][0]
    assert trial["identities"] == [
        {"authority": "nci", "identifier": source_trial["nci_id"]},
        {"authority": "clinicaltrials.gov", "identifier": source_trial["nct_id"]},
    ]
    assert trial["official_title"] == source_trial["official_title"]
    assert trial["phases"] == [source_trial["phase"]]
    assert trial["phase"] == source_trial["phase"]
    assert trial["summary"] == source_trial["brief_summary"]
    assert trial["section_states"] == {
        "arms": "present",
        "eligibility": "present",
        "outcomes": "unavailable",
        "references": "unavailable",
        "contacts": "unavailable",
        "locations": "unavailable",
    }
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


def test_nci_clinical_trial_conformance_case_21_public_success() -> None:
    response = _case21_response()
    rows = _case21_rows(response)
    expected_descriptions = [row["description"] for _, row in rows]
    expected_ids = [source_index + 1 for source_index, _ in rows]
    expected_classifications = [
        "inclusion" if row["inclusion_indicator"] else "exclusion" for _, row in rows
    ]

    json_stdout, _ = _case21_command(response, True)
    markdown, _ = _case21_command(response, False)
    criteria = json.loads(json_stdout)["eligibility"]["criteria"]
    assert len(criteria) == len(expected_descriptions) == 36
    assert [row["id"] for row in criteria] == expected_ids
    assert [row["description"] for row in criteria] == expected_descriptions
    assert [
        row["classification"]["kind"] for row in criteria
    ] == expected_classifications
    assert Counter(row["description"] for row in criteria) == Counter(
        expected_descriptions
    )

    normalized_markdown = "\n".join(
        line.rstrip() for line in markdown.replace("\r\n", "\n").splitlines()
    )
    prior = -1
    for description in expected_descriptions:
        normalized_description = "\n".join(
            line.rstrip() for line in description.replace("\r\n", "\n").splitlines()
        )
        position = normalized_markdown.find(normalized_description)
        assert position > prior
        assert normalized_markdown.count(normalized_description) == 1
        prior = position
    assert normalized_markdown.count("### Inclusion Criteria") == 1
    assert normalized_markdown.count("### Exclusion Criteria") == 1

    changed_description = deepcopy(response)
    changed_description["data"][0]["eligibility"]["unstructured"][0][  # type: ignore[index]
        "description"
    ] = "CASE21 DESCRIPTION SENTINEL"
    changed_json, _ = _case21_command(changed_description, True)
    changed_markdown, _ = _case21_command(changed_description, False)
    changed_criteria = json.loads(changed_json)["eligibility"]["criteria"]
    assert changed_criteria[0]["description"] == "CASE21 DESCRIPTION SENTINEL"
    assert (
        sum(
            row["description"] == "CASE21 DESCRIPTION SENTINEL"
            for row in changed_criteria
        )
        == 1
    )
    assert changed_markdown.count("CASE21 DESCRIPTION SENTINEL") == 1

    changed_order = deepcopy(response)
    changed_order["data"][0]["eligibility"]["unstructured"][0][  # type: ignore[index]
        "display_order"
    ] = 2
    changed_order["data"][0]["eligibility"]["unstructured"][1][  # type: ignore[index]
        "display_order"
    ] = 1
    changed_order_rows = _case21_rows(changed_order)
    changed_order_json, _ = _case21_command(changed_order, True)
    changed_order_markdown, _ = _case21_command(changed_order, False)
    changed_order_markdown = "\n".join(
        line.rstrip()
        for line in changed_order_markdown.replace("\r\n", "\n").splitlines()
    )
    changed_order_criteria = json.loads(changed_order_json)["eligibility"]["criteria"]
    assert [row["id"] for row in changed_order_criteria] == [
        source_index + 1 for source_index, _ in changed_order_rows
    ]
    assert changed_order_criteria[0]["id"] == 2
    prior = -1
    for _, row in changed_order_rows:
        description = "\n".join(
            line.rstrip()
            for line in row["description"].replace("\r\n", "\n").splitlines()
        )
        position = changed_order_markdown.find(description)
        assert position > prior
        prior = position

    changed_classification = deepcopy(response)
    changed_classification["data"][0]["eligibility"]["unstructured"][0][  # type: ignore[index]
        "inclusion_indicator"
    ] = False
    changed_classification_json, _ = _case21_command(changed_classification, True)
    changed_classification_markdown, _ = _case21_command(changed_classification, False)
    assert (
        json.loads(changed_classification_json)["eligibility"]["criteria"][0][
            "classification"
        ]["kind"]
        == "exclusion"
    )
    assert changed_classification_markdown.index("### Exclusion Criteria") < (
        changed_classification_markdown.index("### Inclusion Criteria")
    )


def test_nci_clinical_trial_conformance_case_21_public_absence() -> None:
    response = _case21_response()
    recorded_descriptions = [row["description"] for _, row in _case21_rows(response)]
    for absent_value in ("missing", "null"):
        absent = deepcopy(response)
        eligibility = absent["data"][0]["eligibility"]  # type: ignore[index]
        if absent_value == "missing":
            eligibility.pop("unstructured")
        else:
            eligibility["unstructured"] = None
        json_stdout, _ = _case21_command(absent, True)
        markdown, _ = _case21_command(absent, False)
        assert json.loads(json_stdout)["eligibility"]["criteria"] is None
        assert "### Inclusion Criteria" not in markdown
        assert "### Exclusion Criteria" not in markdown
        assert all(description not in markdown for description in recorded_descriptions)


def test_nci_clinical_trial_conformance_case_21_public_malformed() -> None:
    response = _case21_response()
    requested_identifier = "NCT90000021"
    provider_sentinel = "CASE21 PROVIDER SENTINEL"
    criterion_sentinel = "CASE21 CRITERION SENTINEL"
    response["data"][0]["nct_id"] = requested_identifier  # type: ignore[index]
    response["data"][0]["brief_title"] = provider_sentinel  # type: ignore[index]
    response["data"][0]["eligibility"]["unstructured"][0][  # type: ignore[index]
        "description"
    ] = criterion_sentinel

    malformed = []
    malformed_response = deepcopy(response)
    malformed_response["data"][0]["eligibility"] = True  # type: ignore[index]
    malformed.append(("eligibility is not an object", malformed_response))

    malformed_response = deepcopy(response)
    malformed_response["data"][0]["eligibility"][  # type: ignore[index]
        "unstructured"
    ] = {}
    malformed.append(("unstructured is not an array", malformed_response))

    missing = object()
    for member, replacements in {
        "description": [missing, None, 7, " "],
        "display_order": [missing, None, "1", 1.5, 2**64],
        "inclusion_indicator": [missing, None, "true"],
    }.items():
        for replacement in replacements:
            malformed_response = deepcopy(response)
            criterion = malformed_response["data"][0]["eligibility"][  # type: ignore[index]
                "unstructured"
            ][0]
            criterion.pop(member, None)
            if replacement is not missing:
                criterion[member] = replacement
            label = "missing" if replacement is missing else repr(replacement)
            malformed.append((f"criterion {member}={label}", malformed_response))

    for label, malformed_response in malformed:
        result = _run_nci_detail(
            json.dumps(malformed_response).encode(),
            "--json",
            "get",
            "trial",
            requested_identifier,
            "--source",
            "nci",
            "eligibility",
        )
        assert result.returncode == 1, (label, result.stdout, result.stderr)
        assert result.stderr == "", (label, result.stderr)
        payload = json.loads(result.stdout)
        assert set(payload) == {"error", "_meta"}, (label, payload)
        assert payload["error"] == {
            "code": "api",
            "message": "API request to NCI Clinical Trials Search failed.",
            "source": "NCI Clinical Trials Search",
            "recovery": "Retry the remote source.",
        }, (label, payload)
        assert payload["_meta"] == {"not_found": False}, (label, payload)
        output = result.stdout + result.stderr
        for sentinel in (
            provider_sentinel,
            criterion_sentinel,
            "case21-credential-sentinel",
            requested_identifier,
        ):
            assert sentinel not in output, (label, sentinel, output)
