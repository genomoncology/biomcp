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
    assert report["audited_files"] == 86
    assert report["classified_files"] == 86
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


@pytest.mark.parametrize(
    ("receipt", "body", "expected_error"),
    [
        (
            {
                "provider": "Example Provider",
                "request": "https://example.test/v1/record/42",
                "captured_at": "2026-08-03T00:00:00Z",
                "sha256": "placeholder",
                "minimization_or_redaction": "none; bytes are unmodified",
            },
            b'{"record": 42}\n',
            "provider_origin_statement",
        ),
        (
            {
                "provider": "Example Provider",
                "request": "https://example.test/v1/record/42",
                "captured_at": "2026-08-03T00:00:00Z",
                "sha256": "0" * 64,
                "minimization_or_redaction": "none; bytes are unmodified",
                "provider_origin_statement": "Bytes were recorded from Example Provider before minimization.",
            },
            b'{"record": 42}\n',
            "sha256",
        ),
    ],
)
def test_real_capture_receipts_reject_missing_fields_and_byte_drift(
    tmp_path: Path,
    receipt: dict[str, str],
    body: bytes,
    expected_error: str,
) -> None:
    source_root = tmp_path / "sources"
    payload = source_root / "example" / "record.json"
    payload.parent.mkdir(parents=True)
    payload.write_bytes(body)

    if receipt["sha256"] == "placeholder":
        receipt["sha256"] = hashlib.sha256(body).hexdigest()

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

    result = _audit(source_root)

    assert result.returncode != 0
    assert expected_error in result.stderr
