from __future__ import annotations

import json
import os
from pathlib import Path
import shlex
import subprocess
from urllib.parse import parse_qs, urlparse


REPO_ROOT = Path(__file__).resolve().parents[1]
FIXTURE_SETUP = (
    REPO_ROOT / "spec/fixtures/setup-ctgov-intervention-alias-spec-fixture.sh"
)
FIXTURE_CLEANUP = (
    REPO_ROOT / "spec/fixtures/cleanup-ctgov-intervention-alias-spec-fixture.sh"
)
FIXTURE_ENV = REPO_ROOT / ".cache/spec-ctgov-intervention-alias-env"


def _read_exports(path: Path) -> dict[str, str]:
    exports: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        key, value = line.removeprefix("export ").split("=", 1)
        exports[key] = shlex.split(value)[0]
    return exports


def test_combined_geo_and_eligibility_filters_fetch_one_detail_projection() -> None:
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

        request_log = Path(fixture_env["BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"])
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
    subprocess.run(["bash", str(FIXTURE_SETUP), str(REPO_ROOT)], check=True)
    try:
        fixture_env = _read_exports(FIXTURE_ENV)
        env = os.environ | fixture_env
        env["BIOMCP_MYCHEM_BASE"] = fixture_env[
            "BIOMCP_CTGOV_INTERVENTION_ALIAS_MYCHEM_BASE"
        ]
        biomcp_bin = env.get("BIOMCP_BIN", str(REPO_ROOT / "target/spec/biomcp"))
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

        request_log = Path(fixture_env["BIOMCP_CTGOV_INTERVENTION_ALIAS_REQUEST_LOG"])
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
