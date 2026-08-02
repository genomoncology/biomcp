"""Behavioral contracts for the operator-only variant-article canary."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CANARY = REPO_ROOT / "spec/fixtures/run-variant-articles-live-canary.sh"
G5_CANARY = REPO_ROOT / "spec/fixtures/run-g5-v2-identity-live-canary.sh"


def write_fake_binary(path: Path, marker: Path) -> None:
    path.write_text(
        f"""#!/usr/bin/env python3
import json
from pathlib import Path

Path({str(marker)!r}).touch()
print(json.dumps({{
    "complete": True,
    "results": [{{"pmid": "19142183", "provenance": {{"retrieval_routes": ["pubtator_variant"]}}}}],
    "debug_plan": {{
        "routes": [{{"route": "pubtator_variant", "terminal_state": "complete"}}],
        "provider_queries": [{{"route": "pubtator_variant", "query_template_version": "fixture"}}],
        "candidate_trace": {{"candidates": []}},
    }},
}}))
""",
        encoding="utf-8",
    )
    path.chmod(0o755)


def write_g5_fake_binary(
    path: Path,
    *,
    misattribute_uncalled_provider: bool = False,
    malformed_source_status: bool = False,
    inconsistent_work_allocation: bool = False,
    provider_incomplete: bool = False,
    internal_incomplete: bool = False,
    negative_status_against_ok_call: bool = False,
    stop_detail_without_stopped_route: bool = False,
) -> None:
    source_status = (
        {"exact_lexical": "complete"}
        if malformed_source_status
        else [{
            "route": "exact_lexical",
            "source": "internal" if internal_incomplete else "semanticscholar",
            "status": (
                "not_attempted"
                if internal_incomplete
                else "degraded"
                if provider_incomplete
                else "unavailable"
                if misattribute_uncalled_provider
                else "ok"
            ),
        }]
    )
    if negative_status_against_ok_call:
        source_status[0]["status"] = "degraded"
    if stop_detail_without_stopped_route:
        source_status[0]["detail"] = "one or more aliases stopped before the route bound"
    item_work_consumed = 1 if inconsistent_work_allocation or provider_incomplete else 0
    exact_work_consumed = 1 if provider_incomplete else 0
    complete = not (provider_incomplete or internal_incomplete)
    routes = (
        [{
            "route": "exact_lexical",
            "providers": [{
                "source": "semanticscholar",
                "calls": 1,
                "status": "ok" if negative_status_against_ok_call else "degraded",
            }],
        }]
        if provider_incomplete or negative_status_against_ok_call
        else []
    )
    content = """#!/usr/bin/env python3
import json
import sys

positives = {
    "apc-grch38": "12901799",
    "atm-grch38": "32918381",
    "palb2-grch38": "39999518",
    "mlh1-grch38": "20864636",
}
items = []
for request in json.load(sys.stdin):
    request_id = request["request_id"]
    pmid = positives.get(request_id, "1")
    item = {
        "request_id": request_id,
        "resolution": {"status": "resolved"},
        "results": [{
            "pmid": pmid,
            "routes": ["exact_lexical"],
            "matched_aliases": [f'{request["gene"]} {request["coding"]}'],
            "identity": {"status": "confirmed"},
        }],
        "source_status": SOURCE_STATUS,
        "complete": COMPLETE,
        "truncated": False,
        "error": None,
        "debug_plan": {
            "verification": {},
            "provider_queries": [],
            "budgets": {"item": {"consumed": ITEM_WORK_CONSUMED}},
            "work_allocation": {
                "discovery": {"consumed": 0},
                "exact_lexical": {"item": {"consumed": EXACT_WORK_CONSUMED}},
                "identity_verification": {"item": {"consumed": 0}},
            },
            "routes": ROUTES,
        },
    }
    if request_id in {"atm-grch38", "palb2-grch38"}:
        item["canonical_equivalence"] = {
            "status": "confirmed",
            "complete": True,
            "exhaustive": True,
            "applicable_identity_count": 2,
            "caid": "CA1",
            "observations": [
                {"basis": basis, "status": "resolved", "caid": "CA1", "source": "clingen_car", "comparison_complete": True, "provider_response_sha256": "a" * 64}
                for basis in ("transcript_coding", "genomic")
            ],
        }
    items.append(item)
print(json.dumps({"items": items}))
"""
    path.write_text(
        content.replace("SOURCE_STATUS", repr(source_status))
        .replace("ITEM_WORK_CONSUMED", repr(item_work_consumed))
        .replace("EXACT_WORK_CONSUMED", repr(exact_work_consumed))
        .replace("COMPLETE", repr(complete))
        .replace("ROUTES", repr(routes)),
        encoding="utf-8",
    )
    path.chmod(0o755)


def run_canary(binary: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", str(CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=env,
        text=True,
    )


def test_g5_canary_reports_its_authoritative_verify_role(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    assert completed.returncode == 0
    assert (
        json.loads(completed.stdout)["identity_readiness"][
            "authoritative_verify_treats_g5_as_hard"
        ]
        is True
    )


def test_g5_canary_reports_internal_misattribution_for_an_uncalled_provider(
    tmp_path: Path,
) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, misattribute_uncalled_provider=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert payload["identity_diagnostics"]["internal_misattributions"] == [
        "apc-grch38"
    ]


def test_g5_canary_allows_recorded_provider_incompleteness(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, provider_incomplete=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 0
    assert payload["identity_diagnostics"]["incomplete_results"] == []


def test_g5_canary_rejects_negative_provider_status_against_its_recorded_ok_call(
    tmp_path: Path,
) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, negative_status_against_ok_call=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert "apc-grch38" in payload["identity_diagnostics"][
        "route_status_contradictions"
    ]


def test_g5_canary_rejects_stop_detail_without_stopped_route(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, stop_detail_without_stopped_route=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert "apc-grch38" in payload["identity_diagnostics"][
        "route_status_contradictions"
    ]


def test_g5_canary_rejects_internal_unperformed_work(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, internal_incomplete=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert "apc-grch38" in payload["identity_diagnostics"]["incomplete_results"]


def test_g5_canary_rejects_inconsistent_work_allocation(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, inconsistent_work_allocation=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert (
        payload["identity_readiness"][
            "work_allocation_is_consistent_with_budgets_and_recorded_calls"
        ]
        is False
    )


def test_g5_canary_rejects_malformed_source_status(tmp_path: Path) -> None:
    binary = tmp_path / "biomcp"
    write_g5_fake_binary(binary, malformed_source_status=True)

    completed = subprocess.run(
        ["bash", str(G5_CANARY), str(REPO_ROOT)],
        capture_output=True,
        check=False,
        env=os.environ | {"BIOMCP_BIN": str(binary)},
        text=True,
    )

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert "apc-grch38" in payload["identity_diagnostics"]["schema_parse_failures"]


def test_live_canary_preflight_prints_safe_json_before_invoking_binary(
    tmp_path: Path,
) -> None:
    marker = tmp_path / "called"
    binary = tmp_path / "biomcp"
    write_fake_binary(binary, marker)
    env = os.environ | {"BIOMCP_BIN": str(binary)}
    env.pop("NCBI_API_KEY", None)
    env.pop("S2_API_KEY", None)
    env.pop("UMLS_API_KEY", None)

    completed = run_canary(binary, env)

    assert completed.returncode == 1
    assert json.loads(completed.stdout)["preflight"]["missing"]
    assert not marker.exists()


def test_live_canary_rejects_found_pmids_without_binary_trace_attribution(
    tmp_path: Path,
) -> None:
    marker = tmp_path / "called"
    binary = tmp_path / "biomcp"
    write_fake_binary(binary, marker)
    env = os.environ | {
        "BIOMCP_BIN": str(binary),
        "NCBI_API_KEY": "test",
        "S2_API_KEY": "test",
        "UMLS_API_KEY": "test",
    }

    completed = run_canary(binary, env)

    payload = json.loads(completed.stdout)
    assert completed.returncode == 1
    assert payload["expected_pmid_route_diagnostics_are_binary_attributed"] is False
    assert any(
        row["pmid"] == "19142183" and row["found"] and not row["candidate_routes"]
        for row in payload["expected_pmid_diagnostics"]
    )
    assert marker.exists()
