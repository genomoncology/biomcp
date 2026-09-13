from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

import pytest

ROOT = Path(__file__).resolve().parents[1]
WEBSITE = ROOT / "website"
GENERATOR = WEBSITE / "generate.py"
EXPECTED_REVISION = "991f9fe16b14208e184a6a9a370d5ed7b708dfae"
EXPECTED_BUNDLE_SHA256 = (
    "874989aa405aae4b505f74e13d0f85189692f526e84c84fc29a45e8bc2690854"
)


def _module():
    spec = importlib.util.spec_from_file_location("biodata_site_generator", GENERATOR)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def test_adoption_pin_bundle_and_dependency_agree() -> None:
    manifest = json.loads((WEBSITE / "biodata-adoption.json").read_text())
    bundle = WEBSITE / manifest["bundle_path"]
    assert manifest == {
        "catalog_format": 1,
        "biodata_revision": EXPECTED_REVISION,
        "bundle_path": "catalog/v1/clinical-trial.bundle.json",
        "bundle_sha256": EXPECTED_BUNDLE_SHA256,
    }
    assert hashlib.sha256(bundle.read_bytes()).hexdigest() == EXPECTED_BUNDLE_SHA256
    cargo = (ROOT / "Cargo.toml").read_text()
    lock = (ROOT / "Cargo.lock").read_text()
    assert f'rev = "{EXPECTED_REVISION}"' in cargo
    assert f"?rev={EXPECTED_REVISION}#{EXPECTED_REVISION}" in lock


def test_generated_outputs_are_current_and_cover_the_catalog() -> None:
    result = subprocess.run(
        [sys.executable, GENERATOR, "--check"], cwd=ROOT, text=True, capture_output=True
    )
    assert result.returncode == 0, result.stderr
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    page = (WEBSITE / "src/content/docs/biodata/models/clinical-trial.md").read_text()
    module = _module()
    assert all(
        module.safe_text(field["name"]) in page
        for model in bundle["models"]
        for field in model["fields"]
    )
    assert all(module.safe_text(item["id"]) in page for item in bundle["relationships"])
    assert all(
        module.safe_text(item["explanation"]) in page
        for item in bundle["relationships"]
    )
    assert all(
        module.safe_text(item["qualification"]) in page for item in bundle["crosswalks"]
    )
    assert "correspondence only" in page
    for label in ("provisional", "implemented", "unsupported"):
        assert f"**{label}**" in page


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
    ):
        assert (WEBSITE / "public" / relative).is_file()


@pytest.mark.parametrize(
    ("mutation", "message"),
    [
        (lambda d: d.update(format_version=2), "unsupported catalog format"),
        (
            lambda d: d["models"].append(copy.deepcopy(d["models"][0])),
            "duplicate model identity",
        ),
        (
            lambda d: d["relationships"][0].update(target="model:Missing"),
            "dangling relationship target",
        ),
        (
            lambda d: d["crosswalks"][0].update(local_path=["model:Missing"]),
            "dangling crosswalk locator",
        ),
        (
            lambda d: d["crosswalks"][0].update(canonical_url="http://example.test"),
            "HTTPS",
        ),
        (
            lambda d: d["artifacts"][0].update(path="../escape.json"),
            "unsafe artifact path",
        ),
        (lambda d: d["artifacts"][0].update(content="altered"), "artifact digest"),
        (
            lambda d: d["support"][3].update(executable_proof="proof:fiction"),
            "support contract",
        ),
        (
            lambda d: d["crosswalks"][3].update(
                implementation={"operation": "fake", "executable_proof": None}
            ),
            "executable proof",
        ),
    ],
)
def test_nearest_invalid_catalog_inputs_are_rejected(mutation, message: str) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    mutation(bundle)
    with pytest.raises(module.GenerationError, match=message):
        module.validate_catalog(bundle)


def test_duplicate_json_members_are_rejected() -> None:
    module = _module()
    with pytest.raises(module.GenerationError, match="duplicate JSON member"):
        module.parse_json_strict('{"format_version":1,"format_version":1}')


@pytest.mark.parametrize(
    "hostile_url",
    [
        "https://safe.example/x) [run](javascript:alert(1)",
        "https://safe.example/x\n<script>alert(1)</script>",
        "https://safe.example/white space",
        "https://safe.example/control\x1f",
        "https://safe.example/back\\slash",
        "https://safe.example/{danger}",
    ],
)
def test_hostile_https_urls_are_rejected_before_rendering(hostile_url: str) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    bundle["crosswalks"][0]["canonical_url"] = hostile_url
    with pytest.raises(module.GenerationError, match="safe canonical HTTPS URL"):
        module.render(bundle)


@pytest.mark.parametrize(
    ("mutation", "message"),
    [
        (
            lambda d: d["support"][0].update(executable_proof="proof:unknown"),
            "support contract",
        ),
        (
            lambda d: d["crosswalks"][0]["implementation"].update(operation="fake"),
            "implementation contract",
        ),
        (
            lambda d: d["crosswalks"][0]["evidence"].update(
                binding_sha256="sha256:" + "0" * 64
            ),
            "crosswalk review contract revision is stale",
        ),
        (
            lambda d: d["crosswalks"][0]["review"].update(
                contract_revision="sha256:" + "0" * 64
            ),
            "crosswalk review contract revision is stale",
        ),
        (
            lambda d: d["relationships"][0].update(
                contract_revision="sha256:" + "0" * 64
            ),
            "relationship contract",
        ),
        (lambda d: d["crosswalks"][0].update(direction="both"), "crosswalk direction"),
        (
            lambda d: d["crosswalks"][0].update(semantic_relationship="broader"),
            "semantic relationship",
        ),
        (
            lambda d: d["crosswalks"][0]["evidence"].update(
                artifact_sha256="not-a-digest"
            ),
            "evidence contract",
        ),
        (
            lambda d: d["crosswalks"][0]["review"].update(outcome="pending"),
            "review contract",
        ),
    ],
)
def test_catalog_claims_must_match_the_pinned_format_one_contract(
    mutation, message: str
) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    mutation(bundle)
    with pytest.raises(module.GenerationError, match=message):
        module.validate_catalog(bundle)


@pytest.mark.parametrize(
    "mutation",
    [
        lambda d: d["relationships"][0].update(
            source="model:ClinicalTrial/field:brief_title"
        ),
        lambda d: d["relationships"][0].update(target="model:ExtensibleCode"),
    ],
)
def test_relationship_revision_binds_valid_source_and_target_contracts(
    mutation,
) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    mutation(bundle)
    with pytest.raises(
        module.GenerationError, match="relationship contract revision is stale"
    ):
        module.validate_catalog(bundle)


@pytest.mark.parametrize(
    "mutation",
    [
        lambda d: d["crosswalks"][0].update(
            local_path=["model:ClinicalTrial/field:brief_title"]
        ),
        lambda d: d["crosswalks"][0].update(qualification="Changed qualification."),
        lambda d: d["crosswalks"][0].update(
            canonical_url="https://safe.example/reviewed"
        ),
        lambda d: d["crosswalks"][0].update(external_authority="Other authority"),
        lambda d: d["crosswalks"][0].update(external_version="2.0.6"),
        lambda d: d["crosswalks"][0].update(external_element="other.element"),
        lambda d: d["crosswalks"][0].update(
            direction="biodata_to_external", semantic_relationship="partial"
        ),
        lambda d: d["crosswalks"][3].update(semantic_relationship="related"),
        lambda d: d["crosswalks"][0]["evidence"].update(
            record_locator="evidence.json::contract=other;field=other.element"
        ),
        lambda d: d["crosswalks"][0]["evidence"].update(
            artifact_url="https://safe.example/evidence"
        ),
        lambda d: d["crosswalks"][0]["evidence"].update(artifact_sha256="0" * 64),
        lambda d: d["crosswalks"][0]["evidence"].update(
            element_locator="other.element"
        ),
        lambda d: d["crosswalks"][0]["evidence"].update(
            binding_sha256="sha256:" + "0" * 64
        ),
    ],
)
def test_crosswalk_review_revision_binds_every_reviewed_fact(mutation) -> None:
    module = _module()
    bundle = json.loads((WEBSITE / "catalog/v1/clinical-trial.bundle.json").read_text())
    mutation(bundle)
    with pytest.raises(
        module.GenerationError, match="crosswalk review contract revision is stale"
    ):
        module.validate_catalog(bundle)


@pytest.mark.parametrize(
    "hostile",
    [
        "<script>alert(1)</script>",
        "[run](javascript:alert(1))",
        "{danger}",
        "# injected",
        "```mdx",
    ],
)
def test_catalog_text_renders_as_inert_text(hostile: str) -> None:
    module = _module()
    rendered = module.safe_text(hostile)
    assert "<script" not in rendered
    assert "javascript:" not in rendered
    assert "{danger}" not in rendered
    assert not rendered.startswith("# ")
    assert "```" not in rendered
