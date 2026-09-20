#!/usr/bin/env python3
"""Validate the pinned BioData catalog and generate the ClinicalTrial reference."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Any
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
sys.path.insert(0, str(ROOT))
import model_reference  # noqa: E402
import publication_reference  # noqa: E402

EXPECTED_REVISION = model_reference.EXPECTED_REVISION
EXPECTED_INPUT_DIGEST = model_reference.EXPECTED_INPUT_DIGEST

EXPECTED_DIGEST = "874989aa405aae4b505f74e13d0f85189692f526e84c84fc29a45e8bc2690854"
PUBLICATION_DIGEST = "8edcaa628b092ff9120c9358d4c0bb1f6fac5dff4d5bf8926a40e0af9a5e3eb7"
# The producer hashes an evidence locator before stripping its optional URL fragment.
# Format 1 does not expose that original locator, so the consumer pins the complete
# emitted evidence table after validating each stored binding's shape.
EXPECTED_EVIDENCE_CONTRACT_DIGEST = (
    "sha256:8f802135025d6d42e38b392c9ff93af88c3402ed3230ecdae511328e8f616f1a"
)
EMPTY_EVIDENCE_CONTRACT_DIGEST = (
    "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
)


class GenerationError(ValueError):
    """The adopted input cannot safely generate the public reference."""


CT_IMPLEMENTATIONS = {
    "clinicaltrials-gov-api-v2 source-to-hub": {
        "proof:ctgov-recorded-nct-id-and-report",
        "proof:ctgov-recorded-brief-title-and-report",
        "proof:ctgov-recorded-overall-status-and-report",
    }
}
CT_SUPPORT_PROOFS = {
    "proof:clinical-trial-document-roundtrip",
    "proof:ctgov-recorded-core-and-report",
    "proof:nci-recorded-core-and-report",
}
PUBLICATION_SUPPORT_PROOFS = {
    "proof:scientific-publication-document-roundtrip",
    "proof:pubtator3-recorded-pmid-projection",
    "proof:europepmc-lite-recorded-pmid-projection",
}

ROOT_SPECS = (
    {
        "slug": "clinical-trial", "root_model": "model:ClinicalTrial",
        "manifest": "biodata-adoption.json",
        "bundle_path": "catalog/v1/clinical-trial.bundle.json",
        "bundle_sha256": EXPECTED_DIGEST, "source_sha256": EXPECTED_INPUT_DIGEST,
        "recorded_input": "public/downloads/biodata/nct02576665-provider-types.json",
        "support_proofs": CT_SUPPORT_PROOFS,
        "evidence_digest": EXPECTED_EVIDENCE_CONTRACT_DIGEST,
        "implementations": CT_IMPLEMENTATIONS,
        "diagram_count": 10, "renderer": model_reference,
        "downloads": {
            "schemas/clinical-trial.schema.json": "clinical-trial.schema.json",
            "schemas/clinical-trial-projection.schema.json": "clinical-trial-projection.schema.json",
            "examples/ctgov-clinical-trial-projection.json": "ctgov-clinical-trial-projection.json",
        },
    },
    {
        "slug": "scientific-publication", "root_model": "model:ScientificPublication",
        "manifest": "biodata-adoption-scientific-publication.json",
        "bundle_path": "catalog/v1/scientific-publication.bundle.json",
        "bundle_sha256": PUBLICATION_DIGEST,
        "source_sha256": publication_reference.EXPECTED_SOURCE_DIGEST,
        "recorded_input": None, "support_proofs": PUBLICATION_SUPPORT_PROOFS,
        "evidence_digest": EMPTY_EVIDENCE_CONTRACT_DIGEST,
        "implementations": {}, "diagram_count": 6, "renderer": publication_reference,
        "downloads": {
            "schemas/scientific-publication.schema.json": "scientific-publication.schema.json",
            "examples/pubtator3-scientific-publication.json": "pubtator3-scientific-publication.json",
        },
    },
)


def _object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise GenerationError(f"duplicate JSON member: {key}")
        result[key] = value
    return result


def parse_json_strict(value: str) -> Any:
    try:
        return json.loads(value, object_pairs_hook=_object)
    except json.JSONDecodeError as error:
        raise GenerationError(f"invalid JSON: {error.msg}") from error


def _unique(items: list[dict[str, Any]], label: str) -> set[str]:
    identities: set[str] = set()
    for item in items:
        identity = item.get("id")
        if not isinstance(identity, str) or not identity:
            raise GenerationError(f"missing {label} identity")
        if identity in identities:
            raise GenerationError(f"duplicate {label} identity: {identity}")
        identities.add(identity)
    return identities


def _contract_digest(value: Any) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode()
    return "sha256:" + hashlib.sha256(encoded).hexdigest()


def _locator_contract(
    locator: str, models: dict[str, dict[str, Any]], fields: dict[str, dict[str, Any]]
) -> str:
    if locator in models:
        shape = [
            [field["name"], field["absence"], field["constraints"]]
            for field in models[locator]["fields"]
        ]
        return _contract_digest(shape)
    if locator in fields:
        field = fields[locator]
        return _contract_digest([field["name"], field["absence"], field["constraints"]])
    raise GenerationError("local contract locator does not resolve")


def _https(value: Any, label: str) -> None:
    if not isinstance(value, str) or not value.isascii():
        raise GenerationError(f"{label} must be a safe canonical HTTPS URL")
    parsed = urlsplit(value)
    if (
        parsed.scheme != "https"
        or not parsed.netloc
        or parsed.username
        or parsed.password
        or not re.fullmatch(
            r"https://[A-Za-z0-9.-]+(?::[0-9]{1,5})?"
            r"(?:/[A-Za-z0-9._~:/?#@!$&*+,;=%-]*)?",
            value,
        )
    ):
        raise GenerationError(f"{label} must be a safe canonical HTTPS URL")


def _safe_path(value: Any) -> PurePosixPath:
    if not isinstance(value, str) or "\\" in value:
        raise GenerationError("unsafe artifact path")
    path = PurePosixPath(value)
    if (
        path.is_absolute()
        or not path.parts
        or any(part in ("", ".", "..") for part in path.parts)
    ):
        raise GenerationError("unsafe artifact path")
    return path


def _spec_for(catalog: dict[str, Any]) -> dict[str, Any]:
    for spec in ROOT_SPECS:
        if spec["root_model"] == catalog.get("root_model"):
            return spec
    raise GenerationError("unsupported catalog root")


def validate_catalog(catalog: dict[str, Any]) -> None:
    if catalog.get("format_version") != 1:
        raise GenerationError("unsupported catalog format")
    models = catalog.get("models")
    if not isinstance(models, list):
        raise GenerationError("models must be a list")
    model_ids = _unique(models, "model")
    locators = set(model_ids)
    for model in models:
        fields = model.get("fields")
        if not isinstance(fields, list):
            raise GenerationError("model fields must be a list")
        field_ids = _unique(fields, "field")
        if any(not item.startswith(f"{model['id']}/field:") for item in field_ids):
            raise GenerationError("field identity does not belong to its model")
        locators.update(field_ids)
    models_by_id = {model["id"]: model for model in models}
    fields_by_id = {field["id"]: field for model in models for field in model["fields"]}
    if catalog.get("root_model") not in model_ids:
        raise GenerationError("missing root model identity")
    spec = _spec_for(catalog)

    relationships = catalog.get("relationships")
    if not isinstance(relationships, list):
        raise GenerationError("relationships must be a list")
    relationship_ids = _unique(relationships, "relationship")
    for item in relationships:
        if item.get("source") not in locators:
            raise GenerationError("dangling relationship source")
        if item.get("target") not in model_ids:
            raise GenerationError("dangling relationship target")
        if item.get("kind") not in {"contains", "asserts"}:
            raise GenerationError("invalid relationship kind")
        if not re.fullmatch(r"sha256:[0-9a-f]{64}", str(item.get("contract_revision"))):
            raise GenerationError("malformed relationship contract revision")
        expected_revision = _contract_digest(
            [
                item["source"],
                _locator_contract(item["source"], models_by_id, fields_by_id),
                item["target"],
                _locator_contract(item["target"], models_by_id, fields_by_id),
                item["kind"],
            ]
        )
        if item["contract_revision"] != expected_revision:
            raise GenerationError("relationship contract revision is stale")
    for model in models:
        for field in model["fields"]:
            if any(
                value not in relationship_ids
                for value in field.get("relationship_ids", [])
            ):
                raise GenerationError("dangling field relationship identity")

    support = catalog.get("support")
    if not isinstance(support, list):
        raise GenerationError("support must be a list")
    _unique(support, "support")
    for item in support:
        proof = item.get("executable_proof")
        if proof is not None and proof not in spec["support_proofs"]:
            raise GenerationError("support contract has an unknown executable proof")
        support_state = (
            item.get("maturity"),
            proof is not None,
            item.get("exclusion") is not None,
        )
        expected_label = {
            ("accepted", True, False): "implemented",
            ("unsettled", True, False): "provisional",
            ("intended", False, False): "planned",
            ("accepted", False, True): "unsupported",
        }.get(support_state)
        if expected_label is None or item.get("label") != expected_label:
            raise GenerationError("support contract is contradictory or incomplete")

    crosswalks = catalog.get("crosswalks")
    if not isinstance(crosswalks, list):
        raise GenerationError("crosswalks must be a list")
    _unique(crosswalks, "crosswalk")
    for item in crosswalks:
        path = item.get("local_path")
        if (
            not isinstance(path, list)
            or not path
            or any(value not in locators for value in path)
        ):
            raise GenerationError("dangling crosswalk locator")
        _https(item.get("canonical_url"), "canonical URL")
        evidence = item.get("evidence")
        if not isinstance(evidence, dict) or set(evidence) != {
            "record_locator",
            "artifact_url",
            "artifact_sha256",
            "element_locator",
            "binding_sha256",
        }:
            raise GenerationError("invalid evidence contract")
        _https(evidence.get("artifact_url"), "evidence URL")
        if not re.fullmatch(r"[0-9a-f]{64}", str(evidence.get("artifact_sha256"))):
            raise GenerationError("invalid evidence contract")
        if not re.fullmatch(
            r"sha256:[0-9a-f]{64}", str(evidence.get("binding_sha256"))
        ):
            raise GenerationError("invalid evidence contract")
        review = item.get("review")
        if (
            not isinstance(review, dict)
            or set(review)
            != {
                "reviewer",
                "outcome",
                "rationale",
                "alternatives",
                "contract_revision",
            }
            or review.get("outcome") != "accepted"
            or not re.fullmatch(
                r"sha256:[0-9a-f]{64}", str(review.get("contract_revision"))
            )
        ):
            raise GenerationError("invalid review contract")
        if item.get("direction") not in {"external_to_biodata", "biodata_to_external"}:
            raise GenerationError("invalid crosswalk direction")
        allowed_meaning = {
            ("external_to_biodata", "equivalent"),
            ("biodata_to_external", "partial"),
            ("biodata_to_external", "related"),
        }
        if (
            item.get("direction"),
            item.get("semantic_relationship"),
        ) not in allowed_meaning:
            raise GenerationError("invalid or ambiguous semantic relationship")
        implementation = item.get("implementation")
        if implementation is not None and (
            not isinstance(implementation, dict)
            or not implementation.get("operation")
            or not implementation.get("executable_proof")
        ):
            raise GenerationError("implementation requires executable proof")
        if implementation is not None and implementation.get(
            "executable_proof"
        ) not in spec["implementations"].get(implementation.get("operation"), ()):
            raise GenerationError("implementation contract is unknown")
        local_contract = [
            _locator_contract(locator, models_by_id, fields_by_id) for locator in path
        ]
        expected_review = _contract_digest(
            [
                path,
                local_contract,
                item["direction"],
                item["semantic_relationship"],
                item["qualification"],
                item["external_authority"],
                item["external_version"],
                item["external_element"],
                item["canonical_url"],
                evidence["record_locator"],
                evidence["artifact_url"],
                evidence["artifact_sha256"],
                evidence["element_locator"],
                evidence["binding_sha256"],
            ]
        )
        if review["contract_revision"] != expected_review:
            raise GenerationError("crosswalk review contract revision is stale")

    evidence_contract = {item["id"]: item["evidence"] for item in crosswalks}
    if _contract_digest(evidence_contract) != spec["evidence_digest"]:
        raise GenerationError("evidence binding does not match the pinned catalog")
    artifacts = catalog.get("artifacts")
    if not isinstance(artifacts, list):
        raise GenerationError("artifacts must be a list")
    _unique(artifacts, "artifact")
    for item in artifacts:
        _safe_path(item.get("path"))
        content = item.get("content")
        if not isinstance(content, str):
            raise GenerationError("artifact content must be text")
        digest = "sha256:" + hashlib.sha256(content.encode()).hexdigest()
        if digest != item.get("sha256"):
            raise GenerationError("artifact digest mismatch")


def validate_recorded_input(catalog: dict[str, Any], recorded_input: bytes | None) -> None:
    spec = _spec_for(catalog)
    receipt = catalog.get("example_receipt")
    if not isinstance(receipt, dict):
        raise GenerationError("missing example receipt")
    if receipt.get("source_sha256") != "sha256:" + spec["source_sha256"]:
        raise GenerationError("catalog recorded input digest does not match accepted pin")
    if spec["recorded_input"] is None:
        if recorded_input is not None:
            raise GenerationError("unpublished recorded input")
        return
    if recorded_input is None or hashlib.sha256(recorded_input).hexdigest() != spec["source_sha256"]:
        raise GenerationError("recorded input digest mismatch")


def render_relationship_svg(catalog: dict[str, Any]) -> str:
    spec = _spec_for(catalog)
    validate_catalog(catalog)
    try:
        return model_reference.relationship_svg(catalog, spec["diagram_count"])
    except ValueError as error:
        raise GenerationError(str(error)) from error


def render(catalog: dict[str, Any]) -> str:
    validate_catalog(catalog)
    return _spec_for(catalog)["renderer"].render(catalog)


def safe_text(value: Any) -> str:
    return model_reference.safe_text(value)


def relationship_line(item: dict[str, Any]) -> str:
    return model_reference.relationship_line(item)


def discovery_contract(catalog: dict[str, Any]) -> dict[str, Any]:
    validate_catalog(catalog)
    return _spec_for(catalog)["renderer"].discovery_contract(catalog)


def _page_outputs(
    spec: dict[str, Any],
    catalog: dict[str, Any],
    bundle_bytes: bytes,
    recorded_input: bytes | None,
) -> tuple[dict[Path, bytes], str, dict[str, Any]]:
    validate_recorded_input(catalog, recorded_input)
    body = render(catalog)
    slug = spec["slug"]
    title = spec["root_model"].removeprefix("model:")
    frontmatter = (
        f"---\ntitle: {title}\n"
        f"description: Generated BioData {title} model reference.\n---\n\n"
    )
    discovery = discovery_contract(catalog)
    result = {
        ROOT / f"src/content/docs/biodata/models/{slug}.md": (frontmatter + body).encode(),
        ROOT / f"public/biodata/models/{slug}.md": body.encode(),
        ROOT / f"public/downloads/biodata/{slug}-v1.bundle.json": bundle_bytes,
        ROOT / f"public/downloads/biodata/{slug}-relationships.svg": render_relationship_svg(catalog).encode(),
        ROOT / f"public/biodata/discovery/{slug}.json": (json.dumps(discovery, indent=2, ensure_ascii=False) + "\n").encode(),
    }
    if spec["recorded_input"] is not None and recorded_input is not None:
        result[ROOT / spec["recorded_input"]] = recorded_input
    for artifact in catalog["artifacts"]:
        result[ROOT / "public/downloads/biodata" / spec["downloads"][artifact["path"]]] = artifact["content"].encode()
    return result, body, discovery


def outputs(
    loaded: list[tuple[dict[str, Any], dict[str, Any], bytes, bytes | None]],
) -> dict[Path, bytes]:
    result: dict[Path, bytes] = {}
    index_entries: list[str] = []
    full_parts: list[str] = []
    for spec, catalog, bundle_bytes, recorded_input in loaded:
        pages, body, discovery = _page_outputs(spec, catalog, bundle_bytes, recorded_input)
        result.update(pages)
        title = spec["root_model"].removeprefix("model:")
        model_url = f"https://biomcp.org/biodata/models/{spec['slug']}/"
        raw_url = f"https://biomcp.org/biodata/models/{spec['slug']}.md"
        route_lines = "\n".join(
            f"- [{name.replace('_', ' ').title()}]({url})"
            for name, url in discovery["routes"].items()
        )
        index_entries.append(f"- [{title}]({model_url})\n- [{title} raw Markdown]({raw_url})\n{route_lines}")
        full_parts.append(f"Source: {model_url}\nRaw: {raw_url}\n\n{route_lines}\n\n{body}")
    result[ROOT / "public/llms.txt"] = (
        "# BioMCP\n\n" + "\n".join(index_entries) + "\n"
    ).encode()
    result[ROOT / "public/llms-full.txt"] = (
        "# BioMCP model reference\n\n" + "\n\n".join(full_parts)
    ).encode()
    return result


def load() -> list[tuple[dict[str, Any], dict[str, Any], bytes, bytes | None]]:
    cargo = (REPO / "Cargo.toml").read_text()
    lock = (REPO / "Cargo.lock").read_text()
    if (
        f'rev = "{EXPECTED_REVISION}"' not in cargo
        or f"?rev={EXPECTED_REVISION}#{EXPECTED_REVISION}" not in lock
    ):
        raise GenerationError("BioData dependency does not match adoption manifest")
    loaded = []
    for spec in ROOT_SPECS:
        manifest = parse_json_strict((ROOT / spec["manifest"]).read_text())
        expected = {"catalog_format": 1, "biodata_revision": EXPECTED_REVISION, "bundle_path": spec["bundle_path"], "bundle_sha256": spec["bundle_sha256"]}
        if manifest != expected:
            raise GenerationError("adoption manifest does not match the accepted pin")
        bundle_path = ROOT / _safe_path(manifest["bundle_path"])
        bundle_bytes = bundle_path.read_bytes()
        if hashlib.sha256(bundle_bytes).hexdigest() != spec["bundle_sha256"]:
            raise GenerationError("adopted bundle digest mismatch")
        catalog = parse_json_strict(bundle_bytes.decode())
        if catalog.get("root_model") != spec["root_model"]:
            raise GenerationError("catalog root does not match the adoption manifest")
        validate_catalog(catalog)
        recorded_input = (ROOT / spec["recorded_input"]).read_bytes() if spec["recorded_input"] else None
        validate_recorded_input(catalog, recorded_input)
        loaded.append((spec, catalog, bundle_bytes, recorded_input))
    return loaded


def stale_outputs(generated: dict[Path, bytes]) -> list[str]:
    stale = [
        str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)
        for path, content in generated.items()
        if not path.is_file() or path.read_bytes() != content
    ]
    for relative in (
        "src/content/docs/biodata/models",
        "public/biodata/models",
        "public/biodata/discovery",
        "public/downloads/biodata",
    ):
        directory = ROOT / relative
        expected = {path for path in generated if path.parent == directory}
        if not expected or not directory.is_dir():
            continue
        unexpected = sorted(
            path
            for path in directory.rglob("*")
            if path.is_file() and path not in expected
        )
        stale.extend(str(path.relative_to(ROOT)) for path in unexpected)
    return stale


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        loaded = load()
        generated = outputs(loaded)
        if args.check:
            stale = stale_outputs(generated)
            if stale:
                raise GenerationError("stale generated output: " + ", ".join(stale))
        else:
            for path, content in generated.items():
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
    except (OSError, KeyError, TypeError, GenerationError) as error:
        print(f"model reference generation failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
