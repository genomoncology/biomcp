from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import xml.etree.ElementTree as ET

import pytest

ROOT = Path(__file__).resolve().parents[1]
WEBSITE = ROOT / "website"
GENERATOR = WEBSITE / "generate.py"
EXPECTED_REVISION = "afb4ff9349f5f47e2a9d23305b0ac0ad1efb647c"
EXPECTED_INPUT_SHA256 = (
    "b579ab9ae785d77c228dde7e8c7a6ec43ade347805a8d6f2c9bcadbcf6303f5e"
)
DIRECT_RELATIONSHIP_COUNT = 10
PIN_CHECKER = WEBSITE / "check-biodata-artifact-pins.py"


def _module():
    spec = importlib.util.spec_from_file_location("biodata_surface_generator", GENERATOR)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_stale_and_deleted_generated_outputs_are_reported(tmp_path: Path) -> None:
    module = _module()
    current = tmp_path / "current.txt"
    current.write_bytes(b"current")
    stale = tmp_path / "stale.txt"
    stale.write_bytes(b"old")
    deleted = tmp_path / "deleted.txt"
    assert module.stale_outputs(
        {current: b"current", stale: b"new", deleted: b"expected"}
    ) == [str(stale), str(deleted)]


def test_unexpected_owned_generated_output_is_reported(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    module = _module()
    monkeypatch.setattr(module, "ROOT", tmp_path)
    owned = tmp_path / "public/downloads/biodata"
    owned.mkdir(parents=True)
    expected = owned / "expected.json"
    expected.write_bytes(b"expected")
    unexpected = owned / "retired.json"
    unexpected.write_bytes(b"stale")
    assert module.stale_outputs({expected: b"expected"}) == [
        "public/downloads/biodata/retired.json"
    ]


def test_diagram_is_deterministic_accessible_and_matches_adjacent_text() -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    direct = [
        item
        for item in bundle["relationships"]
        if item["source"].startswith("model:ClinicalTrial/field:")
    ]
    assert len(direct) == DIRECT_RELATIONSHIP_COUNT
    first = module.render_relationship_svg(bundle)
    assert first == module.render_relationship_svg(copy.deepcopy(bundle))
    root = ET.fromstring(first)
    assert root.attrib["role"] == "img"
    assert root.attrib["aria-labelledby"] == "relationship-title relationship-description"
    assert root.find("{http://www.w3.org/2000/svg}title").text
    assert root.find("{http://www.w3.org/2000/svg}desc").text
    forbidden_tags = {"a", "foreignObject", "script", "style", "use", "image"}
    for element in root.iter():
        assert element.tag.rsplit("}", 1)[-1] not in forbidden_tags
        assert not any(
            name.lower().startswith("on") or name.endswith("href")
            for name in element.attrib
        )
    rows = root.findall(".//{http://www.w3.org/2000/svg}g")
    assert len(rows) == DIRECT_RELATIONSHIP_COUNT
    for row, item in zip(rows, direct, strict=True):
        assert row.attrib == {
            "data-relationship-id": item["id"],
            "data-source": item["source"],
            "data-target": item["target"],
            "data-kind": item["kind"],
        }
        texts = {
            node.attrib["data-role"]: node.text
            for node in row.findall("{http://www.w3.org/2000/svg}text")
        }
        assert texts == {
            "id": item["id"],
            "source": item["source"],
            "target": item["target"],
            "kind": item["kind"],
            "explanation": item["explanation"],
        }
    page = module.render(bundle)
    adjacent = page.split("## Direct relationship text", 1)[1].split(
        "## Component relationships", 1
    )[0]
    assert [line for line in adjacent.splitlines() if line.startswith("- `")] == [
        module.relationship_line(item) for item in direct
    ]


def test_recorded_input_and_recipe_are_bound_to_the_catalog_receipt() -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    recorded = WEBSITE / "public/downloads/biodata/nct02576665-provider-types.json"
    assert hashlib.sha256(recorded.read_bytes()).hexdigest() == EXPECTED_INPUT_SHA256
    module.validate_recorded_input(bundle, recorded.read_bytes())
    page = module.render(bundle)
    required = (
        "transformed historical offline evidence",
        "does not present current clinical information",
        "capture rather than provider processing",
        "Consult ClinicalTrials.gov for current information",
        "2026-09-02T22:50:57.882811Z",
        "provider processing date is not recorded",
        "/protocolSection/contactsLocationsModule/overallOfficials",
        "ClinicalTrials.gov updates daily",
        EXPECTED_INPUT_SHA256,
        EXPECTED_REVISION,
        "ClinicalTrialsGovApiV2::parse",
        "project_clinical_trial",
        "into_digest_assertion",
        "Document::ClinicalTrialProjection",
        "conversion report",
        "https://github.com/genomoncology/biodata/blob/"
        + EXPECTED_REVISION
        + "/tests/fixtures/clinicaltrials-gov-v2/manifest.json",
    )
    for value in required:
        assert module.safe_text(value) in page or value in page
    altered = bytearray(recorded.read_bytes())
    altered[-2] ^= 1
    with pytest.raises(module.GenerationError, match="recorded input digest"):
        module.validate_recorded_input(bundle, bytes(altered))


def test_recorded_example_is_wired_into_the_ordinary_offline_gate() -> None:
    makefile = (ROOT / "Makefile").read_text()
    prepared = makefile.split("test-contracts-prepared:\n", 1)[1].split(
        "\nlint:\n", 1
    )[0]
    command = (
        "tools/run-offline -- cargo run --locked --no-default-features --example "
        "biodata-clinical-trial-recorded -- --check"
    )
    assert prepared.count(command) == 1
    assert prepared.index("pytest tests/") < prepared.index(command)
    assert prepared.index(command) < prepared.index("website/check")


def test_discovery_contract_covers_every_public_surface_and_support_limit() -> None:
    discovery = json.loads(
        (WEBSITE / "public/biodata/discovery/clinical-trial.json").read_text()
    )
    assert discovery["model"] == "model:ClinicalTrial"
    assert {task["question"] for task in discovery["tasks"]} >= {
        "What fields describe a clinical trial?",
        "How is a clinical study related to its component records?",
        "How can I reproduce the recorded ClinicalTrials.gov conversion?",
        "Can BioData convert ClinicalTrial to FHIR or USDM?",
    }
    required_routes = {
        "https://biomcp.org/biodata/models/clinical-trial/",
        "https://biomcp.org/biodata/models/clinical-trial.md",
        "https://biomcp.org/downloads/biodata/clinical-trial-relationships.svg",
        "https://biomcp.org/downloads/biodata/nct02576665-provider-types.json",
        "https://biomcp.org/downloads/biodata/ctgov-clinical-trial-projection.json",
        "https://biomcp.org/downloads/biodata/clinical-trial.schema.json",
        "https://biomcp.org/downloads/biodata/clinical-trial-projection.schema.json",
        "https://biomcp.org/downloads/biodata/clinical-trial-v1.bundle.json",
    }
    assert set(discovery["routes"].values()) == required_routes
    answers = " ".join(task["answer"] for task in discovery["tasks"])
    assert "unsupported" in answers
    assert "FHIR" in answers and "USDM" in answers
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    assert discovery["support"] == bundle["support"]
    assert discovery["crosswalks"] == bundle["crosswalks"]


@pytest.mark.parametrize(
    ("support_id", "maturity", "proof", "exclusion", "label"),
    [
        (
            "support:fhir-r4-adapter",
            "intended",
            None,
            None,
            "planned",
        ),
        (
            "support:cdisc-usdm-4-adapter",
            "unsettled",
            "proof:clinical-trial-document-roundtrip",
            None,
            "provisional",
        ),
    ],
)
def test_discovery_support_answer_follows_valid_catalog_alternates(
    support_id: str,
    maturity: str,
    proof: str | None,
    exclusion: str | None,
    label: str,
) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    support = next(item for item in bundle["support"] if item["id"] == support_id)
    support.update(
        maturity=maturity,
        executable_proof=proof,
        exclusion=exclusion,
        label=label,
    )
    module.validate_catalog(bundle)
    discovery = module.discovery_contract(bundle)
    answer = next(
        task["answer"]
        for task in discovery["tasks"]
        if "FHIR or USDM" in task["question"]
    )
    assert f"{support_id}: {label}" in answer
    assert not (label != "unsupported" and f"{support_id}: unsupported" in answer)


def test_raw_markdown_indexes_downloads_and_routes_share_one_identity() -> None:
    source = (WEBSITE / "src/content/docs/biodata/models/clinical-trial.md").read_text()
    raw = (WEBSITE / "public/biodata/models/clinical-trial.md").read_text()
    assert raw == source.split("---\n", 2)[2].lstrip("\n")
    for name in ("llms.txt", "llms-full.txt"):
        index = (WEBSITE / "public" / name).read_text()
        assert "https://biomcp.org/biodata/models/clinical-trial/" in index
        assert "https://biomcp.org/biodata/models/clinical-trial.md" in index
    for relative in (
        "downloads/biodata/clinical-trial.schema.json",
        "downloads/biodata/clinical-trial-projection.schema.json",
        "downloads/biodata/ctgov-clinical-trial-projection.json",
        "downloads/biodata/clinical-trial-v1.bundle.json",
        "downloads/biodata/clinical-trial-relationships.svg",
        "downloads/biodata/nct02576665-provider-types.json",
        "biodata/discovery/clinical-trial.json",
    ):
        assert (WEBSITE / "public" / relative).is_file()


@pytest.mark.parametrize("label", ["planned", "implemented", "provisional", "unsupported"])
def test_support_labels_remain_catalog_driven(label: str) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    support = copy.deepcopy(bundle["support"][0])
    states = {
        "planned": ("intended", None, None),
        "implemented": ("accepted", "proof:clinical-trial-document-roundtrip", None),
        "provisional": ("unsettled", "proof:clinical-trial-document-roundtrip", None),
        "unsupported": ("accepted", None, "No executable adapter exists."),
    }
    support["maturity"], support["executable_proof"], support["exclusion"] = states[label]
    support["label"] = label
    bundle["support"] = [support]
    module.validate_catalog(bundle)
    assert f"**{label}**" in module.render(bundle)


def test_relationship_changes_change_svg_and_removal_is_rejected() -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    original = module.render_relationship_svg(bundle)
    changed = copy.deepcopy(bundle)
    changed["relationships"][0]["explanation"] += " Changed."
    assert module.render_relationship_svg(changed) != original
    removed = copy.deepcopy(bundle)
    removed["relationships"].pop(0)
    with pytest.raises(module.GenerationError, match="dangling field relationship"):
        module.render_relationship_svg(removed)


def test_ordinary_website_check_enforces_biodata_artifact_pins() -> None:
    website_check = (WEBSITE / "check").read_text(encoding="utf-8")
    assert 'python3 "$root/check-biodata-artifact-pins.py"' in website_check
    completed = subprocess.run(
        [sys.executable, str(PIN_CHECKER), str(WEBSITE)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stderr
    assert completed.stdout == ""
    assert completed.stderr == ""


@pytest.mark.parametrize(
    "relative",
    [
        "public/downloads/biodata/clinical-trial-relationships.svg",
        "public/biodata/discovery/clinical-trial.json",
    ],
)
def test_biodata_artifact_pin_rejects_a_one_byte_mutation(
    tmp_path: Path, relative: str
) -> None:
    for artifact in (
        "public/downloads/biodata/clinical-trial-relationships.svg",
        "public/biodata/discovery/clinical-trial.json",
    ):
        destination = tmp_path / artifact
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(WEBSITE / artifact, destination)
    target = tmp_path / relative
    target.write_bytes(target.read_bytes() + b"\n")

    completed = subprocess.run(
        [sys.executable, str(PIN_CHECKER), str(tmp_path)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 1
    assert completed.stdout == ""
    assert "accepted BioData artifact digest mismatch" in completed.stderr
    assert str(tmp_path) not in completed.stderr


def test_hostile_relationship_text_is_inert_in_svg() -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    hostile = '<script onload="alert(1)">&{mdx}[run](javascript:bad)</script>'
    bundle["relationships"][0]["explanation"] = hostile
    svg = module.render_relationship_svg(bundle)
    ET.fromstring(svg)
    assert hostile not in svg
    assert "<script" not in svg
    assert "javascript:bad" in svg
    assert hostile in "".join(ET.fromstring(svg).itertext())


@pytest.mark.parametrize("invalid", ["\x00", "\ufffe", "\ud800"])
def test_xml_invalid_relationship_text_is_safely_replaced(invalid: str) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    bundle["relationships"][0]["explanation"] = f"before{invalid}after"
    svg = module.render_relationship_svg(bundle)
    root = ET.fromstring(svg)
    assert invalid not in svg
    assert "before\N{REPLACEMENT CHARACTER}after" in "".join(root.itertext())
