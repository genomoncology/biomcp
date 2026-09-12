from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
CHECKER = ROOT / "tools/check-biodata-boundary.py"
URL = "https://github.com/genomoncology/biodata"
REVISION = "4f53541dd8c27ffed9cb0bfdfc240c31e0fa698e"


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _fixture(root: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    _write(
        root / "Cargo.toml",
        f'''[package]
name = "boundary-fixture"
version = "1.0.0-dev.1"

[dependencies]
biodata = {{ git = "{URL}", rev = "{REVISION}" }}
''',
    )
    _write(
        root / "Cargo.lock",
        f"""version = 4

[[package]]
name = "biodata"
version = "0.0.22"
source = "git+{URL}?rev={REVISION}#{REVISION}"
""",
    )
    _write(
        root / "src/boundary.rs",
        """use biodata::{
    Capture, ClinicalTrial, ClinicalTrialArm, ClinicalTrialIntervention, ClinicalTrialArms,
    ClinicalTrialArmInterventionAssignment, ClinicalTrialArmRelationshipError,
    ClinicalTrialEligibility, ClinicalTrialReference, ClinicalTrialSection,
    ClinicalTrialsGovApiV2DetailPlan, ClinicalTrialsGovApiV2Response,
    ClinicalTrialSearchSummary, ClinicalTrialsGovApiV2SearchPage,
    NciCtsV2DetailPlan, NciCtsV2DetailResponse,
    NciCtsV2SearchPage,
    ClinicalTrialSearchFilters, ClinicalTrialsGovApiV2SearchPlan, NciCtsV2SearchPlan,
    ClinicalTrialSiteDirectory, ClinicalTrialContact, ClinicalTrialSite,
    ClinicalTrialProjection, ClinicalTrialSearchProjection,
    ClinicalTrialsGovArtifactRetrieval,
};

struct DetailEnvelope { projection: biodata::ClinicalTrialProjection<biodata::Capture> }
struct SearchEnvelope { projection: biodata::ClinicalTrialSearchProjection<biodata::Capture> }

fn projections() {
    detail.into_projection();
    nci_detail.into_projection();
    search.into_projection();
    nci_search.into_projection();
    let _ = ClinicalTrialAgeBound::to_years(&bound);
}
""",
    )
    subprocess.run(["git", "add", "."], cwd=root, check=True)


def _run(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, CHECKER, "--root", root],
        capture_output=True,
        text=True,
        check=False,
    )


def test_biodata_boundary_accepts_the_dedicated_1_0_line() -> None:
    subprocess.run([sys.executable, CHECKER, "--root", ROOT], check=True)


def test_biodata_boundary_accepts_a_complete_minimal_fixture(tmp_path: Path) -> None:
    _fixture(tmp_path)
    assert _run(tmp_path).returncode == 0


@pytest.mark.parametrize(
    ("old", "new"),
    [
        (REVISION, "0" * 40),
        ('version = "0.0.22"', 'version = "0.0.14"'),
    ],
)
def test_biodata_boundary_rejects_wrong_lock_or_revision(
    tmp_path: Path, old: str, new: str
) -> None:
    _fixture(tmp_path)
    for relative in ("Cargo.toml", "Cargo.lock"):
        path = tmp_path / relative
        path.write_text(
            path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8"
        )
    assert _run(tmp_path).returncode == 1


@pytest.mark.parametrize(
    "addition",
    [
        f'''\n[target.'cfg(unix)'.dependencies]\nother = {{ package = "biodata", git = "{URL}", rev = "{REVISION}" }}\n''',
        "\n[target.'cfg(unix)'.dependencies]\nbiodata = { path = \"../biodata\" }\n",
    ],
)
def test_biodata_boundary_rejects_second_or_nested_dependencies(
    tmp_path: Path, addition: str
) -> None:
    _fixture(tmp_path)
    manifest = tmp_path / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="utf-8") + addition, encoding="utf-8"
    )
    subprocess.run(["git", "add", "Cargo.toml"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


def test_biodata_boundary_rejects_a_path_dependency(tmp_path: Path) -> None:
    _fixture(tmp_path)
    manifest = tmp_path / "Cargo.toml"
    manifest.write_text(
        manifest.read_text(encoding="utf-8").replace(
            f'{{ git = "{URL}", rev = "{REVISION}" }}',
            '{ path = "../biodata" }',
        ),
        encoding="utf-8",
    )
    assert _run(tmp_path).returncode == 1


@pytest.mark.parametrize(
    ("relative", "addition"),
    [
        (
            "Cargo.toml",
            f'''\n[patch."{URL}"]\nbiodata = {{ git = "{URL}", rev = "{REVISION}" }}\n''',
        ),
        (
            ".cargo/config.toml",
            f'''[source."{URL}"]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor"\n''',
        ),
    ],
)
def test_biodata_boundary_rejects_patch_and_source_replacements(
    tmp_path: Path, relative: str, addition: str
) -> None:
    _fixture(tmp_path)
    path = tmp_path / relative
    previous = path.read_text(encoding="utf-8") if path.exists() else ""
    _write(path, previous + addition)
    subprocess.run(["git", "add", relative], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


@pytest.mark.parametrize(
    "declaration",
    [
        "pub struct ClinicalTrialArm { value: String }",
        "pub struct ClinicalTrialCore { value: String }",
        "pub struct NciCtsV2DetailPlan { identity: String }",
        "pub struct CtGovDetailPlan { identity: String }",
        "pub struct CtGovDetailResponse { rows: Vec<String> }",
        "pub struct CtGovSearchParams { value: String }",
        "pub struct NciSearchParams { value: String }",
        "pub enum NciDiseaseFilter { Keyword(String) }",
        "pub struct Trial { title: String }",
        "pub struct TrialIdentity { nct_id: String }",
        "pub struct TrialDesign { arms: Vec<String> }",
        "pub struct TrialSearchResult { title: String }",
    ],
)
def test_biodata_boundary_rejects_retired_declarations_in_new_tracked_rust_files(
    tmp_path: Path, declaration: str
) -> None:
    _fixture(tmp_path)
    _write(tmp_path / "src/new_owner.rs", declaration)
    subprocess.run(["git", "add", "src/new_owner.rs"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


def test_biodata_boundary_rejects_a_second_biomedical_field_owner(
    tmp_path: Path,
) -> None:
    _fixture(tmp_path)
    _write(
        tmp_path / "src/new_owner.rs",
        "pub struct AlternateStudy { brief_title: String, conditions: Vec<String> }",
    )
    subprocess.run(["git", "add", "src/new_owner.rs"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


def test_biodata_boundary_rejects_a_duplicate_document_owner(tmp_path: Path) -> None:
    _fixture(tmp_path)
    _write(
        tmp_path / "src/document.rs",
        "pub struct ProductDocument { label: Option<String>, size_bytes: Option<u64> }",
    )
    subprocess.run(["git", "add", "src/document.rs"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


def test_biodata_boundary_rejects_a_widened_adverse_event_consumer(tmp_path: Path) -> None:
    _fixture(tmp_path)
    _write(
        tmp_path / "src/unrelated.rs",
        "fn consume(value: CtGovAdverseEventStudy) { drop(value); }",
    )
    subprocess.run(["git", "add", "src/unrelated.rs"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1


def test_biodata_boundary_rejects_a_bidirectional_trial_codec(tmp_path: Path) -> None:
    _fixture(tmp_path)
    _write(
        tmp_path / "src/codec.rs",
        "fn decode(value: &[u8]) { TrialEligibilityWire::from_json_bytes(value); }",
    )
    subprocess.run(["git", "add", "src/codec.rs"], cwd=tmp_path, check=True)
    assert _run(tmp_path).returncode == 1
