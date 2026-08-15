from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]


def _module(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


notes = _module("release_notes", "release/release_notes.py")


def test_exact_latest_curated_section_is_extracted(tmp_path: Path) -> None:
    changelog = tmp_path / "CHANGELOG.md"
    changelog.write_text(
        "# Changelog\n\n## Unreleased\n\n## 1.2.3 — 2026-08-15\n\n"
        "### Fixes\n\n- Kept release notes truthful.\n\n## 1.2.2 — 2026-08-01\n\n- Older.\n"
    )

    assert notes.extract_release_notes(changelog, "1.2.3") == (
        "## 1.2.3 — 2026-08-15\n\n### Fixes\n\n- Kept release notes truthful.\n"
    )


@pytest.mark.parametrize(
    "text, message",
    [
        ("# Changelog\n\n## Unreleased\n", "missing"),
        (
            "# Changelog\n\n## 1.2.3 — 2026-08-15\n\n- One.\n\n"
            "## 1.2.3 — 2026-08-16\n\n- Two.\n",
            "duplicate",
        ),
        ("# Changelog\n\n## 1.2.3 — 2026-08-15\n\n## 1.2.2 — 2026-08-01\n", "empty"),
        (
            "# Changelog\n\n## 1.2.4 — 2026-08-15\n\n- Newer.\n\n"
            "## 1.2.3 — 2026-08-01\n\n- Requested.\n",
            "latest curated release section",
        ),
    ],
)
def test_invalid_curated_release_sections_are_rejected(
    tmp_path: Path, text: str, message: str
) -> None:
    changelog = tmp_path / "CHANGELOG.md"
    changelog.write_text(text)
    with pytest.raises(notes.ReleaseNotesError, match=message):
        notes.extract_release_notes(changelog, "1.2.3")


def test_publication_uses_curated_notes_without_generated_fallback() -> None:
    script = (ROOT / "release/publish-versioned.sh").read_text()
    assert "release/release_notes.py" in script
    assert "--notes-file" in script
    assert "--generate-notes" not in script
    subprocess.run(["bash", "-n", "release/publish-versioned.sh"], cwd=ROOT, check=True)


@pytest.mark.parametrize(
    ("schema_version", "candidate_kind"),
    [(1, ""), (2, "development")],
)
def test_publish_script_rejects_unsupported_candidate_before_other_inputs(
    tmp_path: Path, schema_version: int, candidate_kind: str
) -> None:
    manifest = tmp_path / "manifest.json"
    value = {
        "schema_version": schema_version,
        "version": "0.9.0-dev.1",
        "source_sha": "a" * 40,
    }
    if candidate_kind:
        value["candidate_kind"] = candidate_kind
    manifest.write_text(json.dumps(value))
    result = subprocess.run(
        [
            "bash",
            "release/publish-versioned.sh",
            str(manifest),
            str(tmp_path / "missing-candidate"),
            str(tmp_path / "missing-inventory"),
        ],
        cwd=ROOT,
        env={**os.environ, "RUNNER_TEMP": str(tmp_path)},
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 2
    assert "promotion:" in result.stderr


def test_publish_script_fully_validates_before_other_inputs_or_effects(
    tmp_path: Path,
) -> None:
    manifest = tmp_path / "manifest.json"
    manifest.write_text(
        json.dumps(
            {
                "schema_version": 2,
                "source_sha": "a" * 40,
                "version": "0.9.0-dev.1",
                "python_version": "0.9.0.dev1",
                "candidate_kind": "release",
                "stage_run_id": "42",
                "status": "complete",
                "created_at": "2026-08-15T00:00:00Z",
                "gates": {},
                "pins": {"rust": "1.93.1"},
                "signing_policy_sha256": None,
                "artifacts": {},
            }
        )
    )
    result = subprocess.run(
        [
            "bash",
            str(ROOT / "release/publish-versioned.sh"),
            str(manifest),
            str(tmp_path / "missing-candidate"),
            str(tmp_path / "missing-inventory"),
        ],
        cwd=tmp_path,
        env={**os.environ, "RUNNER_TEMP": str(tmp_path)},
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 2
    assert "candidate kind does not match its version pair" in result.stderr
    assert list(tmp_path.iterdir()) == [manifest]
