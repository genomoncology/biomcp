#!/usr/bin/env python3
"""Validate the pinned BioData catalog and generate the ClinicalTrial reference."""

from __future__ import annotations

import argparse
import hashlib
import html
import json
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Any
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent
MANIFEST = ROOT / "biodata-adoption.json"
EXPECTED_REVISION = "991f9fe16b14208e184a6a9a370d5ed7b708dfae"
EXPECTED_DIGEST = "874989aa405aae4b505f74e13d0f85189692f526e84c84fc29a45e8bc2690854"
# The producer hashes an evidence locator before stripping its optional URL fragment.
# Format 1 does not expose that original locator, so the consumer pins the complete
# emitted evidence table after validating each stored binding's shape.
EXPECTED_EVIDENCE_CONTRACT_DIGEST = (
    "sha256:8f802135025d6d42e38b392c9ff93af88c3402ed3230ecdae511328e8f616f1a"
)


class GenerationError(ValueError):
    """The adopted input cannot safely generate the public reference."""


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
    known_support_proofs = {
        "proof:clinical-trial-document-roundtrip",
        "proof:ctgov-recorded-core-and-report",
        "proof:nci-recorded-core-and-report",
    }
    for item in support:
        proof = item.get("executable_proof")
        if proof is not None and proof not in known_support_proofs:
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
        if implementation is not None and (
            implementation.get("operation") != "clinicaltrials-gov-api-v2 source-to-hub"
            or implementation.get("executable_proof")
            not in {
                "proof:ctgov-recorded-nct-id-and-report",
                "proof:ctgov-recorded-brief-title-and-report",
                "proof:ctgov-recorded-overall-status-and-report",
            }
        ):
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
    if _contract_digest(evidence_contract) != EXPECTED_EVIDENCE_CONTRACT_DIGEST:
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


def safe_text(value: Any) -> str:
    text = html.escape(str(value), quote=True)
    text = text.replace("javascript:", "javascript&#58;")
    text = text.replace("\\", "&#92;").replace("`", "&#96;").replace("|", "&#124;")
    for char in "*_{}[]()#+-.!":
        text = text.replace(char, f"\\{char}")
    return " ".join(text.split())


def _constraint(value: Any) -> str:
    serialized = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    )
    return safe_text(serialized)


def render(catalog: dict[str, Any]) -> str:
    validate_catalog(catalog)
    root = next(
        model for model in catalog["models"] if model["id"] == catalog["root_model"]
    )
    lines = [
        "# ClinicalTrial",
        "",
        safe_text(root["explanation"]),
        "",
        f"The stable catalog identity is `{safe_text(root['id'])}`.",
        "",
        "## Support and exclusions",
        "",
    ]
    for item in catalog["support"]:
        detail = item.get("exclusion") or item.get("executable_proof")
        lines.append(
            f"- **{safe_text(item['label'])}** `{safe_text(item['id'])}`: {safe_text(detail)}"
        )
    lines += [
        "",
        "## Fields and absence rules",
        "",
        "| Field | Absence | Description | Constraints |",
        "| --- | --- | --- | --- |",
    ]
    for field in root["fields"]:
        description = field.get("description") or "No separate catalog description."
        lines.append(
            f"| `{safe_text(field['name'])}` | `{safe_text(field['absence'])}` | {safe_text(description)} | `{_constraint(field['constraints'])}` |"
        )
    lines += [
        "",
        "Every listed member follows its stated absence rule. Required nullable members distinguish an explicit null from a missing member. Rust validation remains authoritative.",
        "",
        "## Component models",
        "",
    ]
    for model in catalog["models"]:
        if model["id"] == root["id"]:
            continue
        lines += [
            f"### {safe_text(model['id'].removeprefix('model:'))}",
            "",
            safe_text(model["explanation"]),
            "",
            "| Field | Absence | Description | Constraints |",
            "| --- | --- | --- | --- |",
        ]
        for field in model["fields"]:
            description = field.get("description") or "No separate catalog description."
            lines.append(
                f"| `{safe_text(field['name'])}` | `{safe_text(field['absence'])}` | {safe_text(description)} | `{_constraint(field['constraints'])}` |"
            )
        lines.append("")
    lines += ["## Component relationships", ""]
    for item in catalog["relationships"]:
        lines.append(
            f"- `{safe_text(item['id'])}`: `{safe_text(item['source'])}` {safe_text(item['kind'])} `{safe_text(item['target'])}`. {safe_text(item['explanation'])}"
        )
    lines += [
        "",
        "## Provenance",
        "",
        f"This reference comes from catalog format {catalog['format_version']} and the recorded example `{safe_text(catalog['example_receipt']['fixture'])}`. {safe_text(catalog['example_receipt']['attribution'])}.",
        "",
        "## Reviewed crosswalks",
        "",
        "Crosswalk review and adapter support are separate facts. Rows without implementation evidence describe **correspondence only**, not executable conversion.",
        "",
        "| External contract | Direction | Relationship | Local path | Qualification | Conversion |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for item in catalog["crosswalks"]:
        url = item["canonical_url"]
        conversion = (
            "implemented: " + item["implementation"]["operation"]
            if item["implementation"]
            else "correspondence only; no executable conversion"
        )
        local = " → ".join(item["local_path"])
        label = f"{item['external_authority']} {item['external_version']} {item['external_element']}"
        lines.append(
            f"| [{safe_text(label)}](<{url}>) | `{safe_text(item['direction'])}` | `{safe_text(item['semantic_relationship'])}` | `{safe_text(local)}` | {safe_text(item['qualification'])} | {safe_text(conversion)} |"
        )
    lines += [
        "",
        "## Downloads",
        "",
        "- [ClinicalTrial schema](/downloads/biodata/clinical-trial.schema.json)",
        "- [ClinicalTrialProjection schema](/downloads/biodata/clinical-trial-projection.schema.json)",
        "- [Recorded ClinicalTrials.gov projection](/downloads/biodata/ctgov-clinical-trial-projection.json)",
        "- [Adopted catalog bundle](/downloads/biodata/clinical-trial-v1.bundle.json)",
        "",
        "## Scope",
        "",
        "This page renders only the claims and exclusions in the validated adopted catalog.",
        "",
    ]
    return "\n".join(lines)


def outputs(catalog: dict[str, Any], bundle_bytes: bytes) -> dict[Path, bytes]:
    body = render(catalog)
    frontmatter = "---\ntitle: ClinicalTrial\ndescription: Generated BioData ClinicalTrial model reference.\n---\n\n"
    page = frontmatter + body
    model_url = "https://biomcp.org/biodata/models/clinical-trial/"
    raw_url = "https://biomcp.org/biodata/models/clinical-trial.md"
    result = {
        ROOT / "src/content/docs/biodata/models/clinical-trial.md": page.encode(),
        ROOT / "public/biodata/models/clinical-trial.md": body.encode(),
        ROOT / "public/llms.txt": (
            f"# BioMCP\n\n- [ClinicalTrial]({model_url})\n- [ClinicalTrial raw Markdown]({raw_url})\n"
        ).encode(),
        ROOT / "public/llms-full.txt": (
            f"# BioMCP model reference\n\nSource: {model_url}\nRaw: {raw_url}\n\n{body}"
        ).encode(),
        ROOT / "public/downloads/biodata/clinical-trial-v1.bundle.json": bundle_bytes,
    }
    names = {
        "schemas/clinical-trial.schema.json": "clinical-trial.schema.json",
        "schemas/clinical-trial-projection.schema.json": "clinical-trial-projection.schema.json",
        "examples/ctgov-clinical-trial-projection.json": "ctgov-clinical-trial-projection.json",
    }
    for artifact in catalog["artifacts"]:
        result[ROOT / "public/downloads/biodata" / names[artifact["path"]]] = artifact[
            "content"
        ].encode()
    return result


def load() -> tuple[dict[str, Any], bytes]:
    manifest = parse_json_strict(MANIFEST.read_text())
    expected = {
        "catalog_format": 1,
        "biodata_revision": EXPECTED_REVISION,
        "bundle_path": "catalog/v1/clinical-trial.bundle.json",
        "bundle_sha256": EXPECTED_DIGEST,
    }
    if manifest != expected:
        raise GenerationError("adoption manifest does not match the accepted pin")
    bundle_path = ROOT / _safe_path(manifest["bundle_path"])
    bundle_bytes = bundle_path.read_bytes()
    if hashlib.sha256(bundle_bytes).hexdigest() != EXPECTED_DIGEST:
        raise GenerationError("adopted bundle digest mismatch")
    catalog = parse_json_strict(bundle_bytes.decode())
    validate_catalog(catalog)
    cargo = (REPO / "Cargo.toml").read_text()
    lock = (REPO / "Cargo.lock").read_text()
    if (
        f'rev = "{EXPECTED_REVISION}"' not in cargo
        or f"?rev={EXPECTED_REVISION}#{EXPECTED_REVISION}" not in lock
    ):
        raise GenerationError("BioData dependency does not match adoption manifest")
    return catalog, bundle_bytes


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        catalog, bundle_bytes = load()
        generated = outputs(catalog, bundle_bytes)
        if args.check:
            stale = [
                str(path.relative_to(ROOT))
                for path, content in generated.items()
                if not path.is_file() or path.read_bytes() != content
            ]
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
