from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

import tomllib

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-artifact-fixtures"
BIODATA_BOUNDARY_CHECKER = ROOT / "tools/check-biodata-boundary.py"
MAX_PACKAGE_FILES = 1_302
BIODATA_REVISION = "036dd1e2cb51ccbdece3fc1e4ebb2cf8d2509807"


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
    for required in (
        "docs/sources/gencc.md",
        "src/entities/gene/gencc.rs",
        "src/entities/gene/gencc/tests.rs",
        "src/sources/gencc.rs",
        "src/sources/gencc/model.rs",
        "src/sources/gencc/store.rs",
        "src/sources/gencc/tests.rs",
        "src/sources/mygene/tests/live.rs",
        "tests/test_gencc_docs_contract.py",
    ):
        assert required in paths
    assert "testdata/sources/gencc/submissions-new-odc1.csv" not in paths
    subprocess.run(
        [sys.executable, CHECKER, "--manifest"],
        cwd=ROOT,
        input="\n".join(paths) + "\n",
        text=True,
        check=True,
    )


def test_dedicated_line_passes_the_biodata_boundary() -> None:
    subprocess.run(
        [sys.executable, BIODATA_BOUNDARY_CHECKER, "--root", ROOT], check=True
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
    if os.environ.get("BIOMCP_OFFLINE_NETWORK") == "1":
        assert Path(tempfile.gettempdir()) == Path("/tmp")
        assert Path("/tmp") in tmp_path.parents
    else:
        assert ROOT in tmp_path.parents
        assert ROOT in Path(tempfile.gettempdir()).parents


def test_manifest_has_the_exact_reviewed_biodata_dependency_and_compile_deferral() -> (
    None
):
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    assert cargo["dependencies"]["biodata"] == {
        "git": "https://github.com/genomoncology/biodata",
        "rev": BIODATA_REVISION,
    }
    assert cargo["package"]["metadata"]["biodata-development"] == {
        "extracted-package-compile": "deferred",
        "until": "BioMCP 1.0 complete and used internally",
        "reason": "Cargo removes exact Git dependencies from registry packages",
    }


def test_biodata_owns_the_clinical_trial_eligibility_value_codec() -> None:
    source = (ROOT / "src/entities/trial/eligibility.rs").read_text(encoding="utf-8")
    production = source.split("#[cfg(test)]", maxsplit=1)[0]
    assert "ClinicalTrialEligibility::from_json_bytes" in production
    assert ".to_json()" in production
    assert "NO_LIMIT_RULE" not in production
    assert "UnitWire" not in production


def test_biodata_owns_the_clinical_trial_core_model() -> None:
    source = (ROOT / "src/entities/trial/get.rs").read_text(encoding="utf-8")
    assert "biodata::ClinicalTrialCore" in source
    assert "core.brief_summary()" in source


def test_biodata_owns_the_clinical_trial_reference_value_codec() -> None:
    source = (ROOT / "src/entities/trial/mod.rs").read_text(encoding="utf-8")
    production = source.split(
        "#[derive(Debug, Clone, Serialize, Deserialize)]\npub struct TrialSearchResult",
        maxsplit=1,
    )[0]
    assert "ClinicalTrialReference::from_json_bytes" in production
    assert ".to_json()" in production

    provider = (ROOT / "src/sources/clinicaltrials.rs").read_text(encoding="utf-8")
    detail = (ROOT / "src/entities/trial/get.rs").read_text(encoding="utf-8")
    for retired in ("references_module", "CtGovReference", "CtGovReferencesModule"):
        assert retired not in provider
    assert "protocol.references_module = None" not in detail


def test_biodata_owns_trial_sites_contacts_and_product_projection() -> None:
    model = (ROOT / "src/entities/trial/mod.rs").read_text(encoding="utf-8")
    transform = (ROOT / "src/transform/trial.rs").read_text(encoding="utf-8")
    source = (ROOT / "src/sources/clinicaltrials.rs").read_text(encoding="utf-8")
    for retired in (
        "pub struct TrialContact {",
        "pub struct TrialLocation {",
        "pub struct TrialSiteContact {",
        "struct SiteContactKey {",
        "project_contacts_to_locations",
        "extract_contacts",
        "extract_locations",
        "pub struct CtGovContact {",
    ):
        assert retired not in model + transform + source
    assert "ClinicalTrialSiteDirectory" in model
    assert "TrialContactView<'a>" in model
    assert "TrialLocationView<'a>" in model
    assert "TrialSiteContactView<'a>" in model


def test_artifact_checker_rejects_renamed_fixture_bytes(tmp_path: Path) -> None:
    fixture = next(path for path in (ROOT / "testdata").rglob("*") if path.is_file())
    artifact = tmp_path / "bad-wheel.zip"
    with zipfile.ZipFile(artifact, "w") as archive:
        archive.writestr("renamed-provider-response.bin", fixture.read_bytes())
    result = subprocess.run([sys.executable, CHECKER, artifact], cwd=ROOT, check=False)
    assert result.returncode != 0


def test_artifact_checker_accepts_runtime_only_archive(tmp_path: Path) -> None:
    artifact = tmp_path / "good-wheel.zip"
    with zipfile.ZipFile(artifact, "w") as archive:
        archive.writestr("bin/biomcp", b"runtime")
    subprocess.run([sys.executable, CHECKER, artifact], cwd=ROOT, check=True)
