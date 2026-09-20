from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tomllib
import xml.etree.ElementTree as ET

import pytest

ROOT = Path(__file__).resolve().parents[1]
WEBSITE = ROOT / "website"
GENERATOR = WEBSITE / "generate.py"
EXPECTED_REVISION = "c9938b99bd091ab4bf6da8b909ed239826e5ab6d"
EXPECTED_BUNDLE_SHA256 = (
    "8edcaa628b092ff9120c9358d4c0bb1f6fac5dff4d5bf8926a40e0af9a5e3eb7"
)
EXPECTED_SOURCE_SHA256 = (
    "4ab5cb9a467342d0e7c902a5f6328652c36f67d255c5d3c147b04cfebf0e2f40"
)
MANIFEST = WEBSITE / "biodata-adoption-scientific-publication.json"
BUNDLE = WEBSITE / "catalog/v1/scientific-publication.bundle.json"
FOCUSED_MANIFEST = ROOT / "tools/biodata-1.0-focused.toml"
PRODUCT_PROSE_PROOF = {
    "PubTator detail path": (
        "entities::article::detail::pubtator_surfaces::actual_cli_detail_and_ordered_batches_use_adopted_pubtator",
        "entities::article::detail::pubtator_surfaces::actual_raw_and_typed_mcp_preserve_detail_and_sanitize_refusal",
    ),
    "Europe PMC resolution": (
        "entities::article::detail::europepmc_surfaces::doi_and_pmcid_use_one_bound_detail_with_hint_reuse_and_no_pmid_path",
        "entities::article::detail::europepmc_surfaces::actual_cli_json_markdown_and_both_batches_preserve_europepmc_detail",
        "entities::article::detail::europepmc_surfaces::actual_raw_and_typed_mcp_preserve_adopted_detail_and_sanitize_refusal",
    ),
    "HTTP fallback": (
        "entities::article::detail::tests::actual_pubtator_http_404_retains_europepmc_fallback",
        "entities::article::detail::tests::completed_pubtator_enrichment_survives_later_europepmc_failure",
        "entities::article::detail::tests::europepmc_fallback_keeps_authorship_provenance_and_flag",
    ),
}
PUBLICATION_OUTPUTS = {
    "src/content/docs/biodata/models/scientific-publication.md",
    "public/biodata/models/scientific-publication.md",
    "public/biodata/discovery/scientific-publication.json",
    "public/downloads/biodata/scientific-publication-v1.bundle.json",
    "public/downloads/biodata/scientific-publication-relationships.svg",
    "public/downloads/biodata/scientific-publication.schema.json",
    "public/downloads/biodata/pubtator3-scientific-publication.json",
}


def _module():
    spec = importlib.util.spec_from_file_location("biodata_site_generator", GENERATOR)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _bundle() -> dict:
    return json.loads(BUNDLE.read_text())


def _publication(loaded):
    return next(item for item in loaded if item[0]["slug"] == "scientific-publication")


def test_publication_adoption_manifest_matches_the_exact_dependency() -> None:
    manifest = json.loads(MANIFEST.read_text())
    assert manifest == {
        "catalog_format": 1,
        "biodata_revision": EXPECTED_REVISION,
        "bundle_path": "catalog/v1/scientific-publication.bundle.json",
        "bundle_sha256": EXPECTED_BUNDLE_SHA256,
    }
    assert hashlib.sha256(BUNDLE.read_bytes()).hexdigest() == EXPECTED_BUNDLE_SHA256
    cargo = (ROOT / "Cargo.toml").read_text()
    lock = (ROOT / "Cargo.lock").read_text()
    assert f'rev = "{EXPECTED_REVISION}"' in cargo
    assert f"?rev={EXPECTED_REVISION}#{EXPECTED_REVISION}" in lock


def test_publication_catalog_contract_remains_closed() -> None:
    bundle = _bundle()
    assert bundle["format_version"] == 1
    assert bundle["root_model"] == "model:ScientificPublication"
    assert [model["id"] for model in bundle["models"]] == [
        "model:ScientificPublication",
        "model:PublicationIdentifier",
        "model:PublicationTitle",
        "model:PublicationAbstract",
        "model:JournalAssertion",
        "model:AuthorshipAssertion",
        "model:NamedDateAssertion",
    ]
    assert [
        field["id"] for model in bundle["models"] for field in model["fields"]
    ] == [
        "model:ScientificPublication/field:abstract_text",
        "model:ScientificPublication/field:authorships",
        "model:ScientificPublication/field:identifiers",
        "model:ScientificPublication/field:journals",
        "model:ScientificPublication/field:named_dates",
        "model:ScientificPublication/field:title",
        "model:PublicationIdentifier/field:authority",
        "model:PublicationIdentifier/field:identifier",
        "model:PublicationTitle/field:label",
        "model:PublicationTitle/field:raw",
        "model:PublicationTitle/field:state",
        "model:PublicationAbstract/field:label",
        "model:PublicationAbstract/field:raw",
        "model:PublicationAbstract/field:state",
        "model:JournalAssertion/field:label",
        "model:JournalAssertion/field:raw",
        "model:AuthorshipAssertion/field:completeness",
        "model:AuthorshipAssertion/field:label",
        "model:AuthorshipAssertion/field:ordered_display_names",
        "model:AuthorshipAssertion/field:raw",
        "model:NamedDateAssertion/field:label",
        "model:NamedDateAssertion/field:raw",
    ]
    assert len(bundle["relationships"]) == 6
    assert bundle["support"] == [
        {"id": "support:scientific-publication", "maturity": "unsettled", "executable_proof": "proof:scientific-publication-document-roundtrip", "exclusion": None, "label": "provisional"},
        {"id": "support:pubtator3-pmid-projection", "maturity": "accepted", "executable_proof": "proof:pubtator3-recorded-pmid-projection", "exclusion": None, "label": "implemented"},
        {"id": "support:europepmc-lite-pmid-projection", "maturity": "accepted", "executable_proof": "proof:europepmc-lite-recorded-pmid-projection", "exclusion": None, "label": "implemented"},
        {"id": "support:europepmc-core", "maturity": "accepted", "executable_proof": None, "exclusion": "No Europe PMC CORE plan, adapter, or admitted capture exists; only the recorded LITE PMID row is supported.", "label": "unsupported"},
    ]
    assert bundle["crosswalks"] == []
    assert [
        {key: value for key, value in artifact.items() if key != "content"}
        for artifact in bundle["artifacts"]
    ] == [
        {"id": "schema:scientific-publication", "path": "schemas/scientific-publication.schema.json", "media_type": "application/schema+json", "sha256": "sha256:3cd4928fda50e080ab7e05fff66eedc92dd8240fd86bdd12d96240b73823055b"},
        {"id": "example:pubtator3-scientific-publication", "path": "examples/pubtator3-scientific-publication.json", "media_type": "application/json", "sha256": "sha256:33df8e820b97eb7e7e2d708182ccffc0ed77bc5945f8cef40b22c04549a95639"},
    ]
    assert bundle["example_receipt"] == {
        "fixture": "export_39325770.json",
        "origin": "Exact response body captured directly from the public PubTator3 API under Ian Maurer's authorization on 2026-09-15",
        "license": "The article title and abstract are from © 2024 Levchenko et al., licensed CC BY 4.0 at https://creativecommons.org/licenses/by/4.0/. The three captured annotation text and infons.name values are each COVID-19, which is also present in that licensed abstract; this scoped text determination does not assert provider derivation or general annotation-name licensing. Article CC BY does not blanket-license provider additions; source-generated annotations, identifiers, and technical metadata use the separately recorded factual and provider basis.",
        "attribution": "Levchenko M, Parkin M, McEntyre J, Harrison M (2024), Enabling preprint discovery, evaluation, and analysis with Europe PMC, PLOS ONE 19(9): e0303005, https://doi.org/10.1371/journal.pone.0303005; PubTator3 and the U.S. National Library of Medicine",
        "request": "https://www.ncbi.nlm.nih.gov/research/pubtator3-api/publications/export/biocjson?pmids=39325770",
        "source_sha256": "sha256:" + EXPECTED_SOURCE_SHA256,
        "transformation": "None; the committed response body bytes are untouched",
    }
    module = _module()
    module.validate_catalog(bundle)
    module.validate_recorded_input(bundle, None)


def test_publication_output_set_is_exact_and_carries_no_provider_evidence() -> None:
    module = _module()
    generated = module.outputs(module.load())
    paths = {path.relative_to(WEBSITE).as_posix() for path in generated}
    assert PUBLICATION_OUTPUTS <= paths
    assert not any(
        "export_39325770" in path or "annotation" in path for path in paths
    )
    bundle = _bundle()
    artifacts = {item["path"]: item for item in bundle["artifacts"]}
    assert generated[
        WEBSITE / "public/downloads/biodata/scientific-publication-v1.bundle.json"
    ] == BUNDLE.read_bytes()
    for artifact_path, name in (
        ("schemas/scientific-publication.schema.json",
         "scientific-publication.schema.json"),
        ("examples/pubtator3-scientific-publication.json",
         "pubtator3-scientific-publication.json"),
    ):
        artifact = artifacts[artifact_path]
        assert generated[WEBSITE / "public/downloads/biodata" / name] == artifact[
            "content"
        ].encode()
    example = json.loads(artifacts["examples/pubtator3-scientific-publication.json"]["content"])
    assert example["type"] == "biodata/scientific-publication"
    assert example["value"]["identifiers"] == [
        {"authority": "pmid", "identifier": "39325770"}
    ]


def test_publication_page_renders_every_catalog_claim() -> None:
    module = _module()
    bundle = _bundle()
    page = module.render(bundle)
    for model in bundle["models"]:
        for field in model["fields"]:
            assert module.safe_text(field["name"]) in page
            assert module.safe_text(field["absence"]) in page
    for item in bundle["relationships"]:
        assert module.safe_text(item["id"]) in page
        assert module.safe_text(item["explanation"]) in page
    for item in bundle["support"]:
        assert module.safe_text(item["id"]) in page
        detail = item["executable_proof"] or item["exclusion"]
        assert module.safe_text(detail) in page
    for item in bundle["artifacts"]:
        assert module.safe_text(item["id"]) in page
        assert module.safe_text(item["sha256"]) in page
    assert "unsupported" in page
    assert "No reviewed crosswalks" in page
    assert "explicit absence" in page
    assert "correspondence only" not in page


def test_publication_product_prose_states_behavior_and_limits() -> None:
    module = _module()
    page = module.render(_bundle())
    for required in (
        "PubTator",
        "Europe PMC",
        "transport, fallback, reconciliation, and display",
        "does not prove cross-provider equivalence",
        "outside the shared model",
        "biomcp get article 39325770 --json",
        "illustrative",
    ):
        assert required in page
    assert "export_39325770" not in "\n".join(
        line for line in page.splitlines() if "](" in line
    )
    for forbidden in (
        "recorded live output",
        "complete authorship",
        "annotation semantics",
        "Europe PMC CORE is implemented",
        "CORE support",
    ):
        assert forbidden not in page


def test_publication_product_prose_is_bound_to_focused_product_proof() -> None:
    module = _module()
    page = module.render(_bundle())
    rust = tomllib.loads(FOCUSED_MANIFEST.read_text())["selection"]["rust"]
    for claim, selectors in PRODUCT_PROSE_PROOF.items():
        assert claim in page
        for selector in selectors:
            assert selector in rust


@pytest.mark.parametrize(
    ("mutation", "message"),
    [
        (lambda d: d.update(root_model="model:Invented"), "missing root model"),
        (
            lambda d: d.update(root_model="model:PublicationIdentifier"),
            "unsupported catalog root",
        ),
        (
            lambda d: d["models"].append(copy.deepcopy(d["models"][0])),
            "duplicate model identity",
        ),
        (
            lambda d: d["relationships"][0].update(target="model:Missing"),
            "dangling relationship target",
        ),
        (
            lambda d: d["relationships"][0].update(
                contract_revision="sha256:" + "0" * 64
            ),
            "relationship contract",
        ),
        (
            lambda d: d["support"][0].update(executable_proof="proof:fiction"),
            "support contract",
        ),
        (
            lambda d: d["support"][3].update(label="implemented"),
            "support contract is contradictory",
        ),
        (
            lambda d: d["artifacts"][0].update(path="../escape.json"),
            "unsafe artifact path",
        ),
        (
            lambda d: d["artifacts"][0].update(content="altered"),
            "artifact digest",
        ),
        (
            lambda d: d["crosswalks"].append({"id": "crosswalk:invented"}),
            "dangling crosswalk locator",
        ),
    ],
)
def test_publication_catalog_mutations_are_rejected(mutation, message: str) -> None:
    module = _module()
    bundle = _bundle()
    mutation(bundle)
    with pytest.raises(module.GenerationError, match=message):
        module.validate_catalog(bundle)


def test_publication_receipt_digest_is_pinned_without_recorded_input() -> None:
    module = _module()
    bundle = _bundle()
    module.validate_recorded_input(bundle, None)
    with pytest.raises(module.GenerationError, match="unpublished recorded input"):
        module.validate_recorded_input(bundle, b"raw provider bytes")
    changed = copy.deepcopy(bundle)
    changed["example_receipt"]["source_sha256"] = "sha256:" + "0" * 64
    with pytest.raises(module.GenerationError, match="recorded input digest"):
        module.validate_recorded_input(changed, None)


def test_publication_diagram_is_deterministic_accessible_and_bounded() -> None:
    module = _module()
    bundle = _bundle()
    direct = [
        item
        for item in bundle["relationships"]
        if item["source"].startswith("model:ScientificPublication/field:")
    ]
    assert len(direct) == 6
    first = module.render_relationship_svg(bundle)
    assert first == module.render_relationship_svg(copy.deepcopy(bundle))
    root = ET.fromstring(first)
    assert root.attrib["role"] == "img"
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
    assert [row.attrib["data-relationship-id"] for row in rows] == [
        item["id"] for item in direct
    ]
    removed = copy.deepcopy(bundle)
    removed["relationships"].pop(0)
    with pytest.raises(module.GenerationError, match="dangling field relationship"):
        module.render_relationship_svg(removed)


def test_publication_hostile_text_is_inert_in_markdown_and_svg() -> None:
    module = _module()
    bundle = _bundle()
    hostile = '<script onload="alert(1)">&{mdx}[run](javascript:bad)</script>'
    bundle["relationships"][0]["explanation"] = hostile
    svg = module.render_relationship_svg(bundle)
    ET.fromstring(svg)
    assert "<script" not in svg
    assert hostile in "".join(ET.fromstring(svg).itertext())
    bundle["models"][0]["explanation"] = hostile
    page = module.render(bundle)
    assert "<script" not in page
    assert "javascript:bad" not in page


def test_publication_discovery_and_indexes_cover_the_public_surface() -> None:
    module = _module()
    discovery = module.discovery_contract(_bundle())
    assert discovery["model"] == "model:ScientificPublication"
    assert discovery["crosswalks"] == []
    assert set(discovery["routes"].values()) == {
        "https://biomcp.org/biodata/models/scientific-publication/",
        "https://biomcp.org/biodata/models/scientific-publication.md",
        "https://biomcp.org/downloads/biodata/scientific-publication-relationships.svg",
        "https://biomcp.org/downloads/biodata/scientific-publication.schema.json",
        "https://biomcp.org/downloads/biodata/pubtator3-scientific-publication.json",
        "https://biomcp.org/downloads/biodata/scientific-publication-v1.bundle.json",
    }
    assert not any("export_39325770" in url for url in discovery["routes"].values())
    answers = " ".join(task["answer"] for task in discovery["tasks"])
    assert "unsupported" in answers
    for name in ("llms.txt", "llms-full.txt"):
        index = (WEBSITE / "public" / name).read_text()
        for url in discovery["routes"].values():
            assert url in index
        assert "https://biomcp.org/biodata/models/clinical-trial/" in index


def test_publication_sidebar_entry_and_index_link() -> None:
    config = (WEBSITE / "astro.config.mjs").read_text()
    assert "'biodata/models/scientific-publication'" in config
    index = (WEBSITE / "src/content/docs/index.md").read_text()
    assert "/biodata/models/scientific-publication/" in index
