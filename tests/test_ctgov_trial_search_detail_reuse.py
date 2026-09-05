from __future__ import annotations

from contextlib import contextmanager
import fcntl
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import shlex
import subprocess
import threading
from typing import Iterator
from urllib.parse import parse_qs, urlparse

import pytest


REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURE_SETUP = (
    REPO_ROOT / "spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh"
)
FIXTURE_CLEANUP = (
    REPO_ROOT / "spec/fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh"
)
FIXTURE_ENV = REPO_ROOT / ".cache/spec-ctgov-intervention-alias-env"
FIXTURE_LOCK = REPO_ROOT / ".cache/spec-routine-fixtures.lock"
REFERENCE_BINARY = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/spec/biomcp"))
REFERENCE_CAPTURES = REPO_ROOT / "testdata/sources/ctgov"
REFERENCE_FIELDS = [
    "BriefSummary",
    "BriefTitle",
    "CompletionDate",
    "Condition",
    "EnrollmentCount",
    "InterventionDescription",
    "InterventionName",
    "InterventionOtherName",
    "InterventionType",
    "LeadSponsorName",
    "MaximumAge",
    "MinimumAge",
    "NCTId",
    "OverallStatus",
    "Phase",
    "ReferenceCitation",
    "ReferencePMID",
    "ReferenceType",
    "StartDate",
    "StudyType",
    "WhyStopped",
]


@contextmanager
def _routine_fixture_lock() -> Iterator[None]:
    FIXTURE_LOCK.parent.mkdir(parents=True, exist_ok=True)
    with FIXTURE_LOCK.open("w", encoding="utf-8") as lock_file:
        fcntl.flock(lock_file, fcntl.LOCK_EX)
        yield


def _read_exports(path: Path) -> dict[str, str]:
    exports: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        key, value = line.removeprefix("export ").split("=", 1)
        exports[key] = shlex.split(value)[0]
    return exports


def test_combined_geo_and_eligibility_filters_fetch_one_detail_projection() -> None:
    with _routine_fixture_lock():
        subprocess.run(["bash", str(FIXTURE_SETUP), str(REPO_ROOT)], check=True)
        try:
            fixture_env = _read_exports(FIXTURE_ENV)
            env = os.environ | fixture_env
            biomcp_bin = env.get("BIOMCP_BIN", str(REPO_ROOT / "target/spec/biomcp"))
            completed = subprocess.run(
                [
                    biomcp_bin,
                    "--json",
                    "search",
                    "trial",
                    "--mutation",
                    "SHANK3",
                    "--criteria",
                    "SHANK3-related",
                    "--facility",
                    "Rare Disease Center",
                    "--lat",
                    "42.2808",
                    "--lon",
                    "-83.7430",
                    "--distance",
                    "10",
                    "--limit",
                    "1",
                ],
                check=True,
                capture_output=True,
                env=env,
                text=True,
            )
            payload = json.loads(completed.stdout)
            assert [row["nct_id"] for row in payload["results"]] == ["NCT41300001"]

            request_log = Path(
                fixture_env["BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"]
            )
            detail_requests = [
                line
                for line in request_log.read_text(encoding="utf-8").splitlines()
                if urlparse(line).path == "/api/v2/studies/NCT41300001"
            ]
            assert len(detail_requests) == 1, detail_requests

            fields = parse_qs(urlparse(detail_requests[0]).query)["fields"][0].split(",")
            assert "EligibilityCriteria" in fields
            assert "LocationFacility" in fields
            assert "LocationGeoPoint" in fields
        finally:
            subprocess.run(["bash", str(FIXTURE_CLEANUP), str(REPO_ROOT)], check=False)


def _run_alias_search_with_detail_log(
    extra_args: list[str],
) -> tuple[dict[str, object], list[str], bool]:
    with _routine_fixture_lock():
        subprocess.run(["bash", str(FIXTURE_SETUP), str(REPO_ROOT)], check=True)
        try:
            fixture_env = _read_exports(FIXTURE_ENV)
            env = os.environ | fixture_env
            env["BIOMCP_MYCHEM_BASE"] = fixture_env[
                "BIOMCP_CTGOV_INTERVENTION_ALIAS_MYCHEM_BASE"
            ]
            biomcp_bin = env.get(
                "BIOMCP_BIN", str(REPO_ROOT / "target/spec/biomcp")
            )
            completed = subprocess.run(
                [
                    biomcp_bin,
                    "--json",
                    "search",
                    "trial",
                    "--intervention",
                    "venetoclax",
                    "--criteria",
                    "eligible adults",
                    "--source",
                    "ctgov",
                    *extra_args,
                ],
                check=True,
                capture_output=True,
                env=env,
                text=True,
            )

            request_log = Path(
                fixture_env["BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"]
            )
            request_urls = request_log.read_text(encoding="utf-8").splitlines()
            detail_paths = [
                urlparse(url).path
                for url in request_urls
                if urlparse(url).path.startswith("/api/v2/studies/NCT")
            ]
            traversed_later_page = any(
                "pageToken" in parse_qs(urlparse(url).query) for url in request_urls
            )
            return json.loads(completed.stdout), detail_paths, traversed_later_page
        finally:
            subprocess.run(["bash", str(FIXTURE_CLEANUP), str(REPO_ROOT)], check=False)


def test_alias_fanout_deduplicates_candidates_before_detail_verification() -> None:
    payload, detail_paths, traversed_later_page = _run_alias_search_with_detail_log(
        ["--limit", "5"]
    )

    assert [
        (row["nct_id"], row["matched_intervention_label"]) for row in payload["results"]
    ] == [
        ("NCT51000001", "venetoclax"),
        ("NCT51000002", "Venclexta"),
    ]
    assert traversed_later_page
    assert detail_paths.count("/api/v2/studies/NCT51000001") == 1, detail_paths


def test_alias_fanout_count_deduplicates_before_detail_verification() -> None:
    payload, detail_paths, traversed_later_page = _run_alias_search_with_detail_log(
        ["--count-only"]
    )

    assert payload["total"] is None
    assert traversed_later_page
    assert detail_paths.count("/api/v2/studies/NCT51000001") == 1, detail_paths


@contextmanager
def _reference_trial_server():
    replies = {
        nct_id: (REFERENCE_CAPTURES / filename).read_bytes()
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
            status = 200 if payload is not None else 404
            if isinstance(payload, tuple):
                status, payload = payload
            body = (
                payload
                if isinstance(payload, bytes)
                else json.dumps(payload if payload is not None else {}).encode()
            )
            self.send_response(status)
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


def _run_reference(
    base: str,
    cache: Path,
    nct_id: str,
    section: str | list[str],
    *,
    json_output: bool = True,
):
    return subprocess.run(
        [
            REFERENCE_BINARY,
            "--no-cache",
            *(["--json"] if json_output else []),
            "get",
            "trial",
            nct_id,
            "--source",
            "ctgov",
            *([section] if isinstance(section, str) else section),
        ],
        cwd=REPO_ROOT,
        env=os.environ
        | {"BIOMCP_CTGOV_BASE": base, "BIOMCP_CACHE_DIR": str(cache)},
        text=True,
        capture_output=True,
        timeout=30,
    )


@pytest.mark.parametrize("section", ["references", "all"])
def test_recorded_references_and_empty_result_keep_section_behavior(
    tmp_path: Path, section: str
) -> None:
    with _reference_trial_server() as (base, replies, requests):
        for nct_id in replies:
            result = _run_reference(base, tmp_path, nct_id, section)
            assert result.returncode == 0, result.stderr
            source = (
                json.loads(replies[nct_id])["protocolSection"]
                .get("referencesModule", {})
                .get("references", [])
            )
            expected = [
                {
                    "pmid": row["pmid"],
                    "citation": row["citation"],
                    "reference_type": row["type"],
                }
                for row in source
            ]
            assert json.loads(result.stdout)["references"] == expected
            markdown = _run_reference(
                base, tmp_path, nct_id, section, json_output=False
            )
            assert markdown.returncode == 0, markdown.stderr
            assert f"https://clinicaltrials.gov/study/{nct_id}" in markdown.stdout
            positions = []
            for row in expected:
                entry = next(
                    line
                    for line in markdown.stdout.splitlines()
                    if row["pmid"] in line
                )
                assert row["citation"] in entry
                assert row["reference_type"] in entry
                positions.append(markdown.stdout.index(row["pmid"]))
            assert positions == sorted(positions)
            if not expected:
                assert "No references" in markdown.stdout
        for request in requests:
            parsed = urlparse(request)
            assert parsed.path.startswith("/api/v2/studies/NCT")
            fields = parse_qs(parsed.query)["fields"][0].split(",")
            if section == "references":
                assert fields == REFERENCE_FIELDS
            else:
                assert {"ReferencePMID", "ReferenceType", "ReferenceCitation"} <= set(fields)
                assert "PrimaryOutcomeMeasure" in fields


def test_synthetic_partial_reply_normalizes_and_reflects_changed_references(
    tmp_path: Path,
) -> None:
    with _reference_trial_server() as (base, replies, _requests):
        replies["NCT02576665"] = json.loads(replies["NCT02576665"])
        protocol = replies["NCT02576665"]["protocolSection"]
        for module in (
            "sponsorCollaboratorsModule",
            "designModule",
            "conditionsModule",
        ):
            protocol.pop(module, None)
        protocol["referencesModule"] = {
            "references": [
                {
                    "pmid": " 12345 ",
                    "citation": " Changed Étude α. ",
                    "type": " PRIMARY ",
                },
                {
                    "pmid": None,
                    "citation": " Citation without identifiers. ",
                    "type": " \t ",
                },
                {"pmid": " ", "citation": " Another citation. ", "type": None},
                {"citation": "Missing optional members."},
                {"pmid": "discard-empty", "citation": " \t "},
                {"pmid": "discard-null", "citation": None},
                {"pmid": "discard-missing"},
            ]
        }
        expected = [
            {
                "pmid": "12345",
                "citation": "Changed Étude α.",
                "reference_type": "PRIMARY",
            },
            {"citation": "Citation without identifiers."},
            {"citation": "Another citation."},
            {"citation": "Missing optional members."},
        ]
        result = _run_reference(base, tmp_path, "NCT02576665", "references")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["references"] == expected
        markdown = _run_reference(
            base, tmp_path, "NCT02576665", "references", json_output=False
        )
        assert markdown.returncode == 0, markdown.stderr
        assert "Changed Étude α." in markdown.stdout
        assert "Therapeutic activity" not in markdown.stdout
        assert "discard-" not in markdown.stdout


def test_other_trial_section_and_not_found_still_work(tmp_path: Path) -> None:
    with _reference_trial_server() as (base, _replies, _requests):
        result = _run_reference(base, tmp_path, "NCT02576665", "arms")
        assert result.returncode == 0, result.stderr
        assert json.loads(result.stdout)["arms"]
        missing = _run_reference(base, tmp_path, "NCT00000000", "references")
        assert missing.returncode != 0
        combined = missing.stdout + missing.stderr
        assert "not found" in combined.lower()
        assert "Retry the remote source" in combined
        assert "panicked" not in missing.stderr.lower()


def test_mixed_references_request_keeps_the_legacy_field_set(tmp_path: Path) -> None:
    with _reference_trial_server() as (base, _replies, requests):
        result = _run_reference(
            base, tmp_path, "NCT02576665", ["references", "outcomes"]
        )
        assert result.returncode == 0, result.stderr
        fields = parse_qs(urlparse(requests[-1]).query)["fields"][0].split(",")
        assert "ReferenceCitation" in fields
        assert "PrimaryOutcomeMeasure" in fields


def test_reference_validation_and_http_errors_are_safe(tmp_path: Path) -> None:
    with _reference_trial_server() as (base, replies, _requests):
        original = replies["NCT02576665"]
        scenarios = [
            (b"{", "malformed"),
            (b"[]", "unsupported"),
            (
                b'{"protocolSection":{"identificationModule":{}}}',
                "invalid projection",
            ),
            (
                b'{"protocolSection":{"identificationModule":{"nctId":"NCT00000001"}}}',
                "identity mismatch",
            ),
            (b" " * (8 * 1024 * 1024 + 1), "resource limit"),
        ]
        for body, label in scenarios:
            replies["NCT02576665"] = body
            result = _run_reference(base, tmp_path, "NCT02576665", "references")
            assert result.returncode != 0, label
            combined = result.stdout + result.stderr
            assert "ClinicalTrials.gov" in combined
            assert "NCT00000001" not in combined
            assert "protocolSection" not in combined
            assert "identificationModule" not in combined
            recovery = "Narrow the request" if label == "resource limit" else "Retry the remote source"
            assert recovery in combined
        replies["NCT02576665"] = original

        replies["NCT02576665"] = (500, original)
        result = _run_reference(base, tmp_path, "NCT02576665", "references")
        assert result.returncode != 0
        combined = result.stdout + result.stderr
        assert "ClinicalTrials.gov" in combined
        assert "Retry the remote source" in combined
