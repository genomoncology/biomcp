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
import pytest

pytestmark = [pytest.mark.needs_binary]  # runs cargo; docs-only CI excludes this module

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-artifact-fixtures"
ZERO_COUPLING_CHECKER = ROOT / "tools/check-zero-coupling.py"
OFFLINE = ROOT / "tools/run-offline"
# Ticket 1145 raised the package by five authorized module paths:
# citation evidence traversal, its admission proof, the pure JATS citation
# extractor, its tests, and the article CLI citation test sidecar.
MAX_PACKAGE_FILES = 1_406  # ticket 1299 adds the article backends provider-unit module and the article search status module (extraction demanded by the source-size rail), plus 1,300 base + ticket 1145 five modules + ticket 1142 orcid tests submodule + ticket 1199 cache module and tests submodule + ticket 1200 opencitations source module + ticket 1202 cellosaurus source, cell-line entity, render, CLI, docs, and spec files, minus the retired staged release workflow test, plus ticket 1214 cell-line ChEMBL section module and its tests submodule, plus ticket 1213 gene cell-lines module, its tests submodule, and its markdown template, plus ticket 1205 PharmacoDB source module and its three tests submodules, the shared PharmacoDB entity module and its tests, the cell-line drug_response and drug cell_lines section modules with their tests, the PharmacoDB row renderer and its template, the PharmacoDB source page, and the CLI selectors module split out under the line cap, plus ticket 1226 docs-live gate helper and its contract test, plus ticket 1221 CA-bundle helper and its TLS contract test, plus ticket 1233 changelog coverage helper and its contract test, plus ticket 1234 release-version helper and its contract test, plus ticket 1235 poison-recovery helper, plus ticket 1237 label-warnings renderer test module, plus ticket 1245 wheel glibc-floor script and its test, plus ticket 1246 child-stdio guard and Windows stdio contract, plus ticket 1252 wait helpers, wait-ratchet tool and inventory, and their contract tests, plus ticket 1242 trial count tests module and stale-marker cache tests, plus ticket 1243 outcome probe tests module and worker-drive module (ticket 1251's ADR replaces a retired twin in the packaged set), plus the sdlc review-status contract, plus ticket 1255 external-XML guard ticket 1263 stale-JSON note tests, the 2026-09-27 licensing evidence page, and the ticket 1268 GWAS no-store contract, plus ticket 1278's four packaged files (the CI classify script, the nextest install script, the CI workflow contract test, and the trust-failure chain-walk tests) plus the 1279 classify behavioral test, the 1279 record, ticket 1276's file, records 1280 and 1281's four kids26-carried cell-line records (1202, 1205, 1213, 1214), the should-move-latest script and its test, and ticket 1293's six packaged files (the article search deadline module, its tests module, the CLI diagnostics tests module, and the three article-search-deadline fixture scripts), plus ticket 1301's two packaged files (the variant query-resolution module and the gene-first routing spec page, plus ticket 1296's article-entities spec page, plus ticket 1292's variant input forms spec page, plus ticket 1297's two packaged files (the protein-change resolution spec page and its test module), and one correction: 1292's landing also packaged its variant-input-forms fixture handler edit, plus ticket 1304's WHO Prequalification sync outcome test module, its who-sync reporting module, and the who-sync rendering test module that keep the system CLI files under the line ca, plus ticket 1304's three packaged files (the who-pq sync test module and the split who-sync module and its test module), plus ticket 1300's two packaged files (the drug label sections spec page and the label identity-guard tests module), plus the 1300 fix round's one packaged file (the label section pointer tests module), and the second-fix round adds no packaged file; measured with cargo package --list on the merged tree, including ticket 2024 fix round build-identity wrapper contract test module), plus ticket 2029's mutation identity tests submodule)
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
