from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile

import pytest


ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-artifact-fixtures"
MAX_PACKAGE_FILES = 1_302


def _cargo_package_list() -> list[str]:
    result = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty", "--locked", "--offline"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.splitlines()


def _compile_time_include_invocations(source: str) -> list[str]:
    invocations: list[str] = []
    start_pattern = re.compile(r"include_(?:str|bytes)!\s*\(")
    for match in start_pattern.finditer(source):
        depth = 1
        index = match.end()
        in_string = False
        escaped = False
        while index < len(source) and depth:
            char = source[index]
            if in_string:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    in_string = False
            elif char == '"':
                in_string = True
            elif char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            index += 1
        invocations.append(source[match.start() : index])
    return invocations


def test_cargo_source_package_keeps_the_runtime_boundary() -> None:
    paths = _cargo_package_list()
    assert paths
    assert len(paths) <= MAX_PACKAGE_FILES
    assert not any(path == "testdata" or path.startswith("testdata/") for path in paths)
    for private_root in ("architecture", "sdlc"):
        assert not any(
            path == private_root or path.startswith(f"{private_root}/") for path in paths
        )
    subprocess.run(
        [sys.executable, CHECKER, "--manifest"],
        cwd=ROOT,
        input="\n".join(paths) + "\n",
        text=True,
        check=True,
    )


def test_packaged_rust_has_no_private_compile_time_includes() -> None:
    violations: list[str] = []
    for relative in _cargo_package_list():
        if not relative.endswith(".rs"):
            continue
        source = (ROOT / relative).read_text(encoding="utf-8")
        for invocation in _compile_time_include_invocations(source):
            if "architecture/" in invocation or "sdlc/" in invocation:
                violations.append(f"{relative}: {invocation}")
    assert not violations, "private compile-time includes:\n" + "\n".join(violations)


def test_python_contract_temporary_paths_stay_in_worktree(tmp_path: Path) -> None:
    assert ROOT in tmp_path.parents
    assert ROOT in Path(tempfile.gettempdir()).parents


@pytest.mark.skip(
    reason="Requires the public BioData release milestone; development uses an exact Git revision"
)
def test_verified_package_compiles_focused_identity_test_after_extraction(
    tmp_path: Path,
) -> None:
    assert ROOT in tmp_path.parents
    subprocess.run(
        [
            "cargo",
            "package",
            "--allow-dirty",
            "--locked",
            "--offline",
            "--no-verify",
        ],
        cwd=ROOT,
        check=True,
    )
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    package = cargo["package"]
    archive = ROOT / "target" / "package" / f"{package['name']}-{package['version']}.crate"
    assert archive.is_file()

    extract_root = tmp_path / "source-package"
    extract_root.mkdir()
    with tarfile.open(archive, mode="r:gz") as crate:
        crate.extractall(extract_root, filter="data")
    package_root = extract_root / f"{package['name']}-{package['version']}"

    subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            str(package_root / "Cargo.toml"),
            "--locked",
            "--offline",
            "--no-default-features",
            "--test",
            "package_build_identity",
        ],
        cwd=package_root,
        env=os.environ | {"CARGO_TARGET_DIR": str(ROOT / "target")},
        check=True,
    )
    assert not (package_root / "target").exists()


def test_artifact_checker_rejects_renamed_fixture_bytes(tmp_path: Path) -> None:
    fixture = next(path for path in (ROOT / "testdata").rglob("*") if path.is_file())
    artifact = tmp_path / "bad-wheel.zip"
    with zipfile.ZipFile(artifact, "w") as archive:
        archive.writestr("renamed-provider-response.bin", fixture.read_bytes())
    result = subprocess.run([sys.executable, CHECKER, artifact], cwd=ROOT)
    assert result.returncode != 0


def test_artifact_checker_accepts_runtime_only_archive(tmp_path: Path) -> None:
    artifact = tmp_path / "good-wheel.zip"
    with zipfile.ZipFile(artifact, "w") as archive:
        archive.writestr("bin/biomcp", b"runtime")
    subprocess.run([sys.executable, CHECKER, artifact], cwd=ROOT, check=True)
