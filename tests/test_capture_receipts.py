from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
AUDIT_TOOL = REPO_ROOT / "tools" / "check-source-capture-receipts.py"
SOURCES_ROOT = REPO_ROOT / "testdata" / "sources"


def _audit(source_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(AUDIT_TOOL), "--root", str(source_root), "--json"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_clingen_live_replacements_have_receipted_manifest_summary_and_detail_captures() -> None:
    manifest = json.loads((SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8"))
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }

    assert classifications.get("clingen_cspec/atm-manifest.json") == "real_and_receipted"
    assert classifications.get("clingen_cspec/atm-gn020-1.5.1.json") == "real_and_receipted"
    assert classifications.get("clingen_erepo/apc-summary.json") == "real_and_receipted"
    assert classifications.get("clingen_erepo/apc-detail.json") == "real_and_receipted"


def test_clingen_car_and_ldh_live_replacements_have_receipted_captures() -> None:
    manifest = json.loads((SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8"))
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }

    expected_paths = {
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g.json",
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g-empty.json",
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g-malformed.json",
        "clingen_ldh/ca288251-medium.json",
        "clingen_ldh/ca288251-medium-empty.json",
        "clingen_ldh/ca288251-pmc8710334-direct.json",
        "clingen_ldh/ca288251-pmc8710334-direct-malformed.json",
    }

    assert {path for path in expected_paths if classifications.get(path) == "real_and_receipted"} == expected_paths


def test_article_663_source_contract_captures_are_receipted() -> None:
    manifest = json.loads((SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8"))
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }
    expected_paths = {
        "europepmc/pmc3040717-supplementary-not-open-access.xml",
        "europepmc/search_pmid_20516115.json",
        "ncbi_efetch/pmc3040717.xml",
        "pmc_article/pmc3040717-supplementary-tables-pow.html",
        "pmc_article/pmc3040717.html",
        "pmc_oa/pmc3040717-not-open-access.xml",
        "pmc_oa/pmc3040717-versions.xml",
        "pmc_oa/pmc3040717.1.json",
        "pmc_oa/pmc3040717.1.xml",
        "pubtator/export_20516115.json",
        "semantic_scholar/pmid20516115-batch.json",
        "semantic_scholar/pmid20516115-citations.json",
        "semantic_scholar/pmid20516115-recommendations.json",
        "semantic_scholar/pmid20516115-references.json",
    }

    assert {path for path in expected_paths if classifications.get(path) == "real_and_receipted"} == expected_paths


def test_seven_variant_article_corpus_maps_each_landmark_to_receipted_decoded_evidence() -> None:
    manifest = json.loads((SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8"))
    receipts = {
        entry["path"]: entry["receipt"]
        for entry in manifest["entries"]
        if entry["path"].startswith("variant_articles_683/")
        and entry["classification"] == "real_and_receipted"
    }
    map_data = json.loads(
        (SOURCES_ROOT / "variant_articles_683/panel-landmark-map.json").read_text(encoding="utf-8")
    )
    expected_landmarks = {
        "32461654", "22799487", "11805335", "11410501", "20516115", "21990146",
        "18033691", "19142183", "19493351", "26951660", "31433521", "17427195",
    }

    assert {record["landmark_pmid"] for record in map_data["landmarks"]} == expected_landmarks
    assert set(map_data["derived_internal_routes"]) == {
        "strict", "pubtator_variant", "exact_lexical", "source_citation", "best_effort_free_text",
    }

    for record in map_data["landmarks"]:
        path = record["capture_path"]
        assert receipts[path]["request"] == record["safe_request"]
        assert hashlib.sha256(record["safe_request"].encode()).hexdigest() == record["request_sha256"]

        body = json.loads((SOURCES_ROOT / path).read_text(encoding="utf-8"))
        if record["provider"] == "pubmed":
            observed_pmids = set(body.get("esearchresult", {}).get("idlist", []))
        else:
            observed_pmids = {
                result["pmid"] for result in body["resultList"]["result"] if "pmid" in result
            }

        assert (record["landmark_pmid"] in observed_pmids) is record["present"]
        if record["present"]:
            assert record["internal_route"] is not None
        else:
            assert record["internal_route"] is None
            assert record["absence_evidence"]["capture_path"] == path

    states = {evidence["state"] for evidence in map_data["state_evidence"]}
    assert {"positive", "empty", "degraded", "not_attempted"} <= states
    degraded = next(evidence for evidence in map_data["state_evidence"] if evidence["state"] == "degraded")
    assert "error" in json.loads((SOURCES_ROOT / degraded["capture_path"]).read_text(encoding="utf-8"))
    assert {
        evidence["route"] for evidence in map_data["state_evidence"] if evidence["state"] == "not_attempted"
    } == {"car", "ldh"}


def test_repository_audit_classifies_every_source_file_and_preserves_erepo_history() -> None:
    result = _audit(SOURCES_ROOT)

    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["classified_files"] == report["audited_files"]
    assert report["confirmed_byte_unfaithful"] == 0
    assert set(report["classifications"]) == {
        "real_and_receipted",
        "synthetic_and_ineligible",
        "pending_verification",
    }
    assert sum(report["classifications"].values()) == report["audited_files"]
    assert any(
        correction["path"] == "clingen_erepo/apc-detail.json"
        and correction["status"] == "recaptured"
        for correction in report["historical_corrections"]
    )


def _valid_receipt(body: bytes) -> dict[str, str]:
    return {
        "provider": "Example Provider",
        "request": "https://example.test/v1/record/42",
        "captured_at": "2026-08-03T00:00:00Z",
        "sha256": hashlib.sha256(body).hexdigest(),
        "minimization_or_redaction": "none; bytes are unmodified",
        "provider_origin_statement": "Bytes were recorded from Example Provider before minimization.",
    }


def _write_real_capture_inventory(source_root: Path, body: bytes, receipt: dict[str, str]) -> None:
    payload = source_root / "example" / "record.json"
    payload.parent.mkdir(parents=True)
    payload.write_bytes(body)
    (source_root / "capture-receipts.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "entries": [
                    {
                        "path": "example/record.json",
                        "classification": "real_and_receipted",
                        "receipt": receipt,
                    }
                ],
                "historical_corrections": [],
            }
        ),
        encoding="utf-8",
    )


@pytest.mark.parametrize(
    "missing_field",
    (
        "provider",
        "request",
        "captured_at",
        "sha256",
        "minimization_or_redaction",
        "provider_origin_statement",
    ),
)
def test_real_capture_receipts_reject_every_missing_required_field(
    tmp_path: Path, missing_field: str
) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    del receipt[missing_field]
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert missing_field in result.stderr


def test_real_capture_receipts_reject_byte_drift(tmp_path: Path) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    receipt["sha256"] = "0" * 64
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert "sha256" in result.stderr


@pytest.mark.parametrize(
    ("field", "value", "error"),
    (
        (
            "request",
            "https://storage.googleapis.com/object?X-Goog-Signature=secret",
            "unsafe",
        ),
        ("request", "https://example.test/record#opaque-fragment", "unsafe"),
        ("captured_at", "2026-08-03 00:00:00Z", "RFC3339 UTC"),
    ),
)
def test_real_capture_receipts_reject_unsafe_request_and_non_rfc3339_timestamp(
    tmp_path: Path, field: str, value: str, error: str
) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    receipt[field] = value
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert error in result.stderr


def test_repository_audit_does_not_ignore_nested_manifest_named_fixture(
    tmp_path: Path,
) -> None:
    source_root = tmp_path / "sources"
    nested_fixture = source_root / "example" / "capture-receipts.json"
    nested_fixture.parent.mkdir(parents=True)
    nested_fixture.write_text('{"provider": "Example"}\n', encoding="utf-8")
    (source_root / "capture-receipts.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "entries": [],
                "historical_corrections": [],
            }
        ),
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "example/capture-receipts.json" in result.stderr
