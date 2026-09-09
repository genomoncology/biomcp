from __future__ import annotations

from pathlib import Path
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-artifact-fixtures"
ZERO_COUPLING_CHECKER = ROOT / "tools/check-zero-coupling.py"
OFFLINE = ROOT / "tools/run-offline"
MAX_PACKAGE_FILES = 1_300
REMOVED_TRIAL_CRATE = "bio" + "data"


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
    assert len(paths) == MAX_PACKAGE_FILES
    assert not any(path == "testdata" or path.startswith("testdata/") for path in paths)
    for private_root in ("architecture", "sdlc"):
        assert not any(
            path == private_root or path.startswith(f"{private_root}/")
            for path in paths
        )
    subprocess.run(
        [sys.executable, CHECKER, "--manifest"],
        cwd=ROOT,
        input="\n".join(paths) + "\n",
        text=True,
        check=True,
    )


def test_verified_source_package_is_zero_coupled_and_compiles_offline(
    tmp_path: Path,
) -> None:
    package_target = tmp_path / "package-target"
    subprocess.run(
        [
            OFFLINE,
            "--",
            "cargo",
            "package",
            "--locked",
            "--offline",
            "--allow-dirty",
            "--target-dir",
            package_target,
        ],
        cwd=ROOT,
        check=True,
    )
    archives = list((package_target / "package").glob("biomcp-cli-*.crate"))
    assert len(archives) == 1
    archive = archives[0]
    with tarfile.open(archive, "r:gz") as source:
        members = source.getmembers()
        assert len(members) == MAX_PACKAGE_FILES
        source.extractall(tmp_path / "unpacked", filter="data")
    subprocess.run(
        [sys.executable, ZERO_COUPLING_CHECKER, "--archive", archive], check=True
    )
    unpacked = tmp_path / "unpacked" / archive.name.removesuffix(".crate")
    env = os.environ | {"CARGO_TARGET_DIR": str(tmp_path / "check-target")}
    subprocess.run(
        [OFFLINE, "--", "cargo", "check", "--locked", "--offline"],
        cwd=unpacked,
        env=env,
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


def test_manifest_has_no_external_trial_crate_or_compile_deferral() -> None:
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    assert REMOVED_TRIAL_CRATE not in cargo["dependencies"]
    assert (
        REMOVED_TRIAL_CRATE
        not in str(cargo.get("package", {}).get("metadata", {})).lower()
    )


def test_biomcp_owns_the_clinical_trial_eligibility_value_codec() -> None:
    source = (ROOT / "src/entities/trial/eligibility.rs").read_text(encoding="utf-8")
    production = source.split("#[cfg(test)]", maxsplit=1)[0]
    assert "ClinicalTrialEligibility::from_json_bytes" in production
    assert ".to_json()" in production
    assert "NO_LIMIT_RULE" in production
    assert "UnitWire" in production


def test_biomcp_owns_the_clinical_trial_reference_value_codec() -> None:
    source = (ROOT / "src/entities/trial/mod.rs").read_text(encoding="utf-8")
    production = source.split(
        "#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TrialSearchResult",
        maxsplit=1,
    )[0]
    assert "fn decode(input: &[u8])" in production
    assert "strict_json::validate" in production

    provider = (ROOT / "src/sources/clinicaltrials.rs").read_text(encoding="utf-8")
    detail = (ROOT / "src/entities/trial/get.rs").read_text(encoding="utf-8")
    for required in ("references_module", "CtGovReference", "CtGovReferencesModule"):
        assert required in provider
    assert "protocol.references_module = None" not in detail


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
