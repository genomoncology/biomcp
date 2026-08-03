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
