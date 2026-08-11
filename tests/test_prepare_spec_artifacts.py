from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess


REPO_ROOT = Path(__file__).resolve().parents[1]
PREPARER = REPO_ROOT / "scripts" / "prepare-spec-artifacts.py"


def _load_preparer():
    spec = importlib.util.spec_from_file_location("prepare_spec_artifacts", PREPARER)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_cargo_artifacts_uses_cargo_reported_executables(tmp_path: Path) -> None:
    executable = tmp_path / "deps" / "health_cli_structure-123"
    lines = [
        "not json",
        json.dumps({"reason": "build-script-executed"}),
        json.dumps(
            {
                "reason": "compiler-artifact",
                "target": {"name": "health_cli_structure"},
                "executable": str(executable),
            }
        ),
    ]

    assert _load_preparer().cargo_artifacts(lines) == {
        "health_cli_structure": executable.resolve()
    }


def test_install_artifact_creates_a_stable_executable_copy(tmp_path: Path) -> None:
    source = tmp_path / "target" / "biomcp-hash"
    source.parent.mkdir()
    source.write_text("binary", encoding="utf-8")
    destination = tmp_path / ".cache" / "spec-artifacts" / "feature-off" / "biomcp"

    installed = _load_preparer().install_artifact(source, destination)

    assert installed == destination.resolve()
    assert destination.read_text(encoding="utf-8") == "binary"
    assert destination.stat().st_mode & 0o111


def test_runner_rejects_a_missing_prepared_artifact(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    scripts = workspace / "scripts"
    scripts.mkdir(parents=True)
    shutil.copy2(REPO_ROOT / "scripts" / "run-specs.sh", scripts / "run-specs.sh")
    (workspace / "Cargo.toml").write_text(
        "[package]\nname='fixture'\nversion='0.1.0'\n"
    )
    preparer = scripts / "prepare-spec-artifacts.py"
    preparer.write_text(
        "import argparse\n"
        "p=argparse.ArgumentParser()\n"
        "p.add_argument('--mode'); p.add_argument('--profile'); p.add_argument('--output'); p.add_argument('--cargo-feature-arg', action='append')\n"
        "a=p.parse_args()\n"
        "names=['BIOMCP_BIN','BIOMCP_SPEC_FEATURE_OFF_BIN','BIOMCP_SPEC_MCP_EXAMPLE_BIN','BIOMCP_SPEC_CARGO_TREE','BIOMCP_SPEC_CARGO_METADATA']\n"
        "open(a.output, 'w').write(''.join(f'export {name}=/missing/{name}\\n' for name in names))\n",
        encoding="utf-8",
    )
    bin_dir = workspace / "bin"
    bin_dir.mkdir()
    mustmatch = bin_dir / "mustmatch"
    mustmatch.write_text(
        "#!/usr/bin/env bash\n"
        "if [[ ${1:-} == --version ]]; then echo 'mustmatch 1.0.0'; fi\n",
        encoding="utf-8",
    )
    mustmatch.chmod(0o755)

    completed = subprocess.run(
        ["bash", "scripts/run-specs.sh", "spec"],
        cwd=workspace,
        env=os.environ
        | {
            "MUSTMATCH_BIN": str(mustmatch),
            "ROUTINE_CARGO_FEATURES": "--no-default-features",
        },
        text=True,
        capture_output=True,
        check=False,
    )

    assert completed.returncode == 1
    assert (
        "prepared spec artifact is missing or empty: "
        "/missing/BIOMCP_SPEC_FEATURE_OFF_BIN"
    ) in completed.stderr
    assert "cargo" not in completed.stderr
