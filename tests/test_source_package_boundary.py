from __future__ import annotations

import os
import re
import runpy
import subprocess
import sys
import tempfile
import tomllib
import zipfile
from collections.abc import Callable, Collection, Mapping
from functools import cache
from pathlib import Path, PurePosixPath

import pytest

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-artifact-fixtures"
BIODATA_BOUNDARY_CHECKER = ROOT / "tools/check-biodata-boundary.py"
BIODATA_REVISION = "bf111ab25f620a8e16cc92518b693f9696940c0b"
REVIEWED_TOP_LEVEL_DIRECTORIES = {
    ".claude-plugin",
    ".github",
    "Formula",
    "benchmarks",
    "bin",
    "docs",
    "examples",
    "release",
    "scripts",
    "skills",
    "spec",
    "src",
    "templates",
    "tests",
    "tools",
}
REQUIRED_ROOT_ENTRIES = {
    ".cargo_vcs_info.json",
    ".dockerignore",
    ".gitattributes",
    ".gitignore",
    ".mcpbignore",
    ".rustfmt.toml",
    ".zenodo.json",
    "AGENTS.md",
    "CHANGELOG.md",
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "Cargo.lock",
    "Cargo.toml",
    "Cargo.toml.orig",
    "Dockerfile",
    "LICENSE",
    "Makefile",
    "README.md",
    "RUN.md",
    "build.rs",
    "deny.toml",
    "install.sh",
    "manifest.json",
    "mkdocs.yml",
    "pyproject.toml",
    "server.json",
    "uv.lock",
}
REQUIRED_PACKAGE_MEMBERS = {
    "docs/sources/gencc.md",
    "src/entities/gene/gencc.rs",
    "src/entities/gene/gencc/tests.rs",
    "src/sources/gencc.rs",
    "src/sources/gencc/model.rs",
    "src/sources/gencc/store.rs",
    "src/sources/gencc/tests.rs",
    "src/sources/mygene/tests/live.rs",
    "tests/test_gencc_docs_contract.py",
}
REQUIRED_AREA_ROOTS = {
    "documentation": "docs",
    "specifications": "spec",
    "examples": "examples",
    "release support": "release",
    "scripts": "scripts",
    "tools": "tools",
    "tests": "tests",
}


def _rust_function(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start : index + 1]
    raise AssertionError(f"unterminated Rust function: {signature}")


@cache
def _cargo_package_list() -> tuple[str, ...]:
    result = subprocess.run(
        ["cargo", "package", "--list", "--allow-dirty", "--locked", "--offline"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return tuple(result.stdout.splitlines())


@cache
def _artifact_fixture_contract() -> tuple[Callable[[bytes], str], set[str]]:
    checker = runpy.run_path(str(CHECKER))
    digest = checker["digest"]
    fixture_digests = checker["fixture_digests"]
    return digest, fixture_digests()


def _tracked_paths_under(*roots: str) -> set[str]:
    result = subprocess.run(
        ["git", "ls-files", "--", *roots],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return set(result.stdout.splitlines())


def _require_package_members(
    paths: set[str], required: Collection[str], purpose: str
) -> None:
    missing = sorted(set(required) - paths)
    assert not missing, f"missing {purpose}: {', '.join(missing)}"


def _validate_source_package(
    package_paths: Collection[str],
    *,
    required_root_entries: Collection[str],
    tracked_source_paths: Collection[str],
    production_compile_time_inputs: Collection[str],
    tracked_skill_paths: Collection[str],
    member_payloads: Mapping[str, bytes],
    forbidden_fixture_digests: Collection[str],
    payload_digest: Callable[[bytes], str],
    required_area_roots: Collection[str] | Mapping[str, str],
    reviewed_top_level_directories: Collection[str],
) -> None:
    paths = set(package_paths)
    assert paths, "source package is empty"

    reviewed_roots = set(required_root_entries)
    reviewed_directories = set(reviewed_top_level_directories)
    for path in sorted(paths):
        parts = PurePosixPath(path).parts
        assert parts, f"invalid empty package path: {path!r}"
        if "testdata" in parts:
            raise AssertionError(f"package contains testdata path component: {path}")
        if parts[0] in {"architecture", "sdlc"}:
            raise AssertionError(f"package contains private root: {path}")
        if len(parts) == 1:
            if path not in reviewed_roots:
                raise AssertionError(f"package contains unreviewed root file: {path}")
        elif parts[0] not in reviewed_directories:
            raise AssertionError(
                f"package contains unreviewed top-level directory: {parts[0]}"
            )

    _require_package_members(paths, reviewed_roots, "required root entries")
    _require_package_members(paths, REQUIRED_PACKAGE_MEMBERS, "named package members")
    _require_package_members(
        paths, production_compile_time_inputs, "production compile-time inputs"
    )
    _require_package_members(paths, tracked_source_paths, "tracked runtime inputs")
    _require_package_members(paths, tracked_skill_paths, "tracked packaged skills")

    area_roots = (
        required_area_roots.values()
        if isinstance(required_area_roots, Mapping)
        else required_area_roots
    )
    for root in area_roots:
        assert any(path.startswith(f"{root}/") for path in paths), (
            f"missing required package area: {root}"
        )

    forbidden = set(forbidden_fixture_digests)
    for path in sorted(paths):
        payload = member_payloads.get(path)
        if payload is None:
            continue
        if payload_digest(payload) in forbidden:
            raise AssertionError(f"package member matches captured fixture bytes: {path}")
        if not path.endswith(".rs"):
            continue
        source = payload.decode("utf-8")
        for invocation in _compile_time_include_invocations(source):
            if "architecture/" in invocation or "sdlc/" in invocation:
                raise AssertionError(
                    f"packaged Rust has private compile-time include: {path}: {invocation}"
                )


def _validate_real_source_package(paths: Collection[str]) -> None:
    tracked_templates = _tracked_paths_under("templates")
    digest, forbidden_fixture_digests = _artifact_fixture_contract()
    member_payloads = {
        path: (ROOT / path).read_bytes()
        for path in paths
        if (ROOT / path).is_file()
    }
    _validate_source_package(
        paths,
        required_root_entries=REQUIRED_ROOT_ENTRIES,
        tracked_source_paths=_tracked_paths_under("src"),
        production_compile_time_inputs={
            "src/cli/list_reference.md",
            "spec/fixtures/vaers/reactions-request.xml",
            *tracked_templates,
        },
        tracked_skill_paths=_tracked_paths_under("skills"),
        member_payloads=member_payloads,
        forbidden_fixture_digests=forbidden_fixture_digests,
        payload_digest=digest,
        required_area_roots=REQUIRED_AREA_ROOTS,
        reviewed_top_level_directories=REVIEWED_TOP_LEVEL_DIRECTORIES,
    )


def _validator_fixture() -> tuple[set[str], dict[str, object]]:
    tracked_source_paths = {"src/main.rs", "src/cli/list_reference.md"}
    production_inputs = {
        "src/cli/list_reference.md",
        "templates/report.md",
        "spec/fixtures/vaers/reactions-request.xml",
    }
    tracked_skills = {"skills/SKILL.md"}
    area_members = {f"{root}/member" for root in REQUIRED_AREA_ROOTS.values()}
    paths = (
        REQUIRED_ROOT_ENTRIES
        | REQUIRED_PACKAGE_MEMBERS
        | tracked_source_paths
        | production_inputs
        | tracked_skills
        | area_members
    )
    arguments: dict[str, object] = {
        "required_root_entries": REQUIRED_ROOT_ENTRIES,
        "tracked_source_paths": tracked_source_paths,
        "production_compile_time_inputs": production_inputs,
        "tracked_skill_paths": tracked_skills,
        "member_payloads": {},
        "forbidden_fixture_digests": set(),
        "payload_digest": lambda payload: payload.hex(),
        "required_area_roots": REQUIRED_AREA_ROOTS,
        "reviewed_top_level_directories": REVIEWED_TOP_LEVEL_DIRECTORIES,
    }
    return paths, arguments


def test_package_validator_allows_an_additional_source_member() -> None:
    paths, arguments = _validator_fixture()
    arguments["member_payloads"] = {
        "src/new_module.rs": b"pub fn new_module() {}"
    }
    _validate_source_package(paths | {"src/new_module.rs"}, **arguments)


def test_package_validator_requires_each_cargo_generated_root_entry() -> None:
    paths, arguments = _validator_fixture()
    for generated in (".cargo_vcs_info.json", "Cargo.toml.orig"):
        with pytest.raises(
            AssertionError, match=rf"required root entries: {re.escape(generated)}"
        ):
            _validate_source_package(paths - {generated}, **arguments)


def test_package_validator_rejects_missing_required_root_entry() -> None:
    paths, arguments = _validator_fixture()
    with pytest.raises(AssertionError, match="missing required root entries: Cargo.toml"):
        _validate_source_package(paths - {"Cargo.toml"}, **arguments)


def test_package_validator_rejects_missing_non_rust_runtime_input() -> None:
    paths, arguments = _validator_fixture()
    arguments["production_compile_time_inputs"] = {
        "templates/report.md",
        "spec/fixtures/vaers/reactions-request.xml",
    }
    with pytest.raises(
        AssertionError,
        match="missing tracked runtime inputs: src/cli/list_reference.md",
    ):
        _validate_source_package(paths - {"src/cli/list_reference.md"}, **arguments)


def test_package_validator_rejects_missing_external_compile_time_input() -> None:
    paths, arguments = _validator_fixture()
    external = "spec/fixtures/vaers/reactions-request.xml"
    with pytest.raises(
        AssertionError, match=f"missing production compile-time inputs: {external}"
    ):
        _validate_source_package(paths - {external}, **arguments)


def test_package_validator_rejects_private_roots_and_nested_testdata() -> None:
    paths, arguments = _validator_fixture()
    with pytest.raises(AssertionError, match="package contains private root: sdlc/note.md"):
        _validate_source_package(paths | {"sdlc/note.md"}, **arguments)
    with pytest.raises(
        AssertionError,
        match="package contains testdata path component: docs/testdata/capture.json",
    ):
        _validate_source_package(paths | {"docs/testdata/capture.json"}, **arguments)


def test_package_validator_rejects_unreviewed_top_level_entries() -> None:
    paths, arguments = _validator_fixture()
    with pytest.raises(
        AssertionError, match="package contains unreviewed root file: SECURITY.md"
    ):
        _validate_source_package(paths | {"SECURITY.md"}, **arguments)
    with pytest.raises(
        AssertionError,
        match="package contains unreviewed top-level directory: crates",
    ):
        _validate_source_package(paths | {"crates/new.rs"}, **arguments)


def test_package_validator_requires_tracked_skills_and_reviewed_areas() -> None:
    paths, arguments = _validator_fixture()
    with pytest.raises(
        AssertionError, match="missing tracked packaged skills: skills/SKILL.md"
    ):
        _validate_source_package(paths - {"skills/SKILL.md"}, **arguments)
    with pytest.raises(AssertionError, match="missing required package area: examples"):
        _validate_source_package(paths - {"examples/member"}, **arguments)


def test_package_validator_rejects_private_rust_compile_time_include() -> None:
    paths, arguments = _validator_fixture()
    source = "src/private_include.rs"
    arguments["member_payloads"] = {
        source: b'const DESIGN: &str = include_str!("../architecture/design.md");'
    }
    with pytest.raises(
        AssertionError,
        match="packaged Rust has private compile-time include: src/private_include.rs",
    ):
        _validate_source_package(paths | {source}, **arguments)


def test_package_validator_rejects_fixture_bytes_under_an_approved_root() -> None:
    paths, arguments = _validator_fixture()
    copied = "docs/copied-provider-response.bin"
    fixture_payload = b"captured provider response"
    arguments["member_payloads"] = {copied: fixture_payload}
    arguments["forbidden_fixture_digests"] = {fixture_payload.hex()}
    with pytest.raises(
        AssertionError,
        match=f"package member matches captured fixture bytes: {copied}",
    ):
        _validate_source_package(paths | {copied}, **arguments)


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
    _validate_real_source_package(paths)
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
        "#[derive(Clone, Serialize, Deserialize)]\npub struct TrialSearchResult",
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


def test_biodata_owns_trial_document_detail_manifest_and_provenance_paths() -> None:
    provider = (ROOT / "src/sources/clinicaltrials.rs").read_text(encoding="utf-8")
    detail = (ROOT / "src/entities/trial/get.rs").read_text(encoding="utf-8")
    documents = (ROOT / "src/entities/trial/documents.rs").read_text(encoding="utf-8")
    migrated = provider + detail + documents
    for retired in (
        "CtGovBiodataDetailResponse",
        "CtGovLargeDocument",
        "document_section",
        "large_documents",
        "map_document",
    ):
        assert retired not in migrated
    migrated_bodies = (
        _rust_function(provider, "pub(crate) fn decode_biodata_detail_response"),
        _rust_function(provider, "pub(crate) async fn get_biodata_detail"),
        _rust_function(documents, "pub async fn trial_documents_manifest"),
        _rust_function(documents, "pub async fn trial_document_bytes"),
    )
    for body in migrated_bodies:
        assert "decode_get_response" not in body
        assert "client.get(" not in body
    product_get = _rust_function(detail, "pub async fn get(")
    ctgov_get = product_get.split("TrialSource::ClinicalTrialsGov =>", maxsplit=1)[1].split(
        "TrialSource::NciCts =>", maxsplit=1
    )[0]
    assert "decode_get_response" not in ctgov_get
    assert "client.get(" not in ctgov_get
    assert ".get_biodata_detail(" in ctgov_get
    assert ".get_biodata_detail(" in migrated_bodies[2]
    assert "ClinicalTrialsGovArtifactDescriptor" in documents
    assert "response.capture()" in documents

    eligibility = (ROOT / "src/entities/trial/search/eligibility.rs").read_text(
        encoding="utf-8"
    )
    adverse_events = (ROOT / "src/entities/adverse_event.rs").read_text(encoding="utf-8")
    transform = (ROOT / "src/transform/trial.rs").read_text(encoding="utf-8")
    verifier = _rust_function(eligibility, "pub(super) async fn verify_detail_filters")
    assert verifier.count("client.get_biodata_detail(&nct_id, &sections).await") == 1
    assert "client.get(" not in verifier

    search_decode = _rust_function(provider, "pub async fn search(")
    assert "Result<ClinicalTrialsGovApiV2SearchPage" in search_decode
    assert "decode_search_response" in search_decode
    fetch = _rust_function(adverse_events, "async fn fetch_ctgov_studies_for_alias")
    assert "Result<Vec<CtGovAdverseEventStudy>" in fetch
    assert "client: &ClinicalTrialsClient" in fetch
    assert ".search_adverse_events(" in fetch
    assert "response.studies" in fetch
    aggregate = _rust_function(adverse_events, "fn trial_adverse_events_from_study_batches")
    assert aggregate.count("CtGovAdverseEventStudy") >= 2
    adverse_production = adverse_events.split("#[cfg(test)]", maxsplit=1)[0]
    outside_named_consumers = adverse_production.replace(fetch, "").replace(aggregate, "")
    outside_named_consumers = outside_named_consumers.replace(
        "CtGovAdverseEventStudy,", ""
    )
    assert "CtGovAdverseEventStudy" not in outside_named_consumers

    migrated = provider + (ROOT / "src/sources/nci_cts.rs").read_text(encoding="utf-8")
    for retired in (
        "CtGovSearchResponse",
        "CtGovStudy",
        "NciSearchResponse",
        "from_ctgov_hit",
        "from_nci_hit",
        "decode_get_response",
    ):
        assert retired not in migrated + transform


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
