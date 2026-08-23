from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tomllib


ROOT = Path(__file__).resolve().parents[1]


def _run_mkdocs(*args: str, cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "uv",
            "run",
            "--project",
            str(ROOT),
            "--no-sync",
            "mkdocs",
            *args,
        ],
        cwd=cwd,
        env=os.environ | {"NO_MKDOCS_2_WARNING": "1"},
        capture_output=True,
        text=True,
        check=False,
    )


def test_docs_dependencies_cannot_resolve_mkdocs_or_material_major_upgrade() -> None:
    project = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
    dependencies = project["project"]["optional-dependencies"]["dev"]

    assert "mkdocs>=1.6,<2" in dependencies
    assert "mkdocs-material>=9.5,<10" in dependencies


def test_routine_docs_gate_silences_the_material_mkdocs_warning() -> None:
    makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
    routine_docs_build = makefile.split("test-contracts-prepared:\n", 1)[1].split(
        "\nlint:", 1
    )[0]

    assert "NO_MKDOCS_2_WARNING=1" in routine_docs_build


def test_committed_docs_lockfile_is_current_offline() -> None:
    completed = subprocess.run(
        ["uv", "lock", "--check", "--offline"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )

    assert completed.returncode == 0, completed.stderr


def test_strict_docs_build_stays_clean_when_warning_is_silenced(tmp_path: Path) -> None:
    completed = _run_mkdocs("build", "--strict", "--site-dir", str(tmp_path / "site"))

    assert completed.returncode == 0, completed.stdout + completed.stderr
    assert "MkDocs 2.0 is incompatible with Material" not in completed.stdout


def test_strict_docs_build_rejects_broken_internal_links(tmp_path: Path) -> None:
    docs = tmp_path / "docs"
    docs.mkdir()
    (docs / "index.md").write_text("[Missing](missing.md)\n", encoding="utf-8")
    (tmp_path / "mkdocs.yml").write_text(
        "site_name: Broken link fixture\ndocs_dir: docs\n",
        encoding="utf-8",
    )

    completed = _run_mkdocs("build", "--strict", cwd=tmp_path)

    assert completed.returncode != 0
    assert "missing.md" in completed.stdout + completed.stderr
