"""Render the one bounded ClinicalTrial documentation surface."""

from __future__ import annotations

import html
import json
from typing import Any

EXPECTED_REVISION = "c9938b99bd091ab4bf6da8b909ed239826e5ab6d"
EXPECTED_INPUT_DIGEST = "b579ab9ae785d77c228dde7e8c7a6ec43ade347805a8d6f2c9bcadbcf6303f5e"
RECEIPT_URL = (
    "https://github.com/genomoncology/biodata/blob/"
    f"{EXPECTED_REVISION}/tests/fixtures/clinicaltrials-gov-v2/manifest.json"
)

def safe_text(value: Any) -> str:
    text = html.escape(str(value), quote=True)
    text = text.replace("javascript:", "javascript&#58;")
    text = text.replace("\\", "&#92;").replace("`", "&#96;").replace("|", "&#124;")
    for char in "*_{}[]()#+-.!":
        text = text.replace(char, f"\\{char}")
    return " ".join(text.split())


def _xml_text(value: Any) -> str:
    def valid(char: str) -> bool:
        codepoint = ord(char)
        return (
            codepoint in {0x9, 0xA, 0xD}
            or 0x20 <= codepoint <= 0xD7FF
            or 0xE000 <= codepoint <= 0xFFFD
            or 0x10000 <= codepoint <= 0x10FFFF
        )

    text = "".join(
        char if valid(char) else "\N{REPLACEMENT CHARACTER}"
        for char in str(value)
    )
    return html.escape(text, quote=True)


def direct_relationships(catalog: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        item
        for item in catalog["relationships"]
        if item["source"].startswith(f"{catalog['root_model']}/field:")
    ]


_COUNT_WORDS = {6: "Six", 10: "Ten"}


def relationship_svg(catalog: dict[str, Any], expected_count: int) -> str:
    root_name = catalog["root_model"].removeprefix("model:")
    relationships = direct_relationships(catalog)
    if (
        expected_count not in _COUNT_WORDS
        or len(relationships) != expected_count
    ):
        raise ValueError(
            f"{root_name} diagram requires {expected_count} direct relationships"
        )
    width = 1600
    row_height = 136
    height = 72 + row_height * len(relationships)
    lines = [
        '<?xml version="1.0" encoding="UTF-8"?>',
        (
            f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" '
            f'height="{height}" viewBox="0 0 {width} {height}" role="img" '
            'aria-labelledby="relationship-title relationship-description">'
        ),
        f'<title id="relationship-title">{root_name} direct relationships</title>',
        (
            f'<desc id="relationship-description">{_COUNT_WORDS[expected_count]} '
            f'direct relationships from {root_name} fields to component models, '
            'in adopted catalog order. Equivalent text follows the image.</desc>'
        ),
        '<rect x="24" y="30" width="250" height="56" rx="8" fill="#eef5ff" stroke="#315c8c"/>',
        f'<text x="149" y="64" text-anchor="middle" font-family="sans-serif" font-size="20">{root_name}</text>',
    ]
    for index, item in enumerate(relationships):
        y = 28 + index * row_height
        middle = y + 31
        lines += [
            f'<line x1="274" y1="58" x2="340" y2="{middle}" stroke="#52687a" stroke-width="2"/>',
            (
                f'<g data-relationship-id="{_xml_text(item["id"])}" '
                f'data-source="{_xml_text(item["source"])}" '
                f'data-target="{_xml_text(item["target"])}" '
                f'data-kind="{_xml_text(item["kind"])}">'
            ),
            f'<rect x="340" y="{y}" width="1236" height="116" rx="8" fill="#ffffff" stroke="#52687a"/>',
            (
                f'<text x="360" y="{y + 21}" font-family="sans-serif" font-size="17" '
                'font-weight="bold" data-role="id">'
                f'{_xml_text(item["id"])}</text>'
            ),
            (
                f'<text x="360" y="{y + 43}" font-family="sans-serif" font-size="14" '
                f'data-role="source">{_xml_text(item["source"])}</text>'
            ),
            (
                f'<text x="360" y="{y + 65}" font-family="sans-serif" font-size="14" '
                f'data-role="target">{_xml_text(item["target"])}</text>'
            ),
            (
                f'<text x="360" y="{y + 87}" font-family="sans-serif" font-size="14" '
                f'data-role="kind">{_xml_text(item["kind"])}</text>'
            ),
            (
                f'<text x="360" y="{y + 109}" font-family="sans-serif" font-size="14" '
                f'data-role="explanation">{_xml_text(item["explanation"])}</text>'
            ),
            "</g>",
        ]
    lines.append("</svg>")
    return "\n".join(lines) + "\n"




def _constraint(value: Any) -> str:
    serialized = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    )
    return safe_text(serialized)


def relationship_line(item: dict[str, Any]) -> str:
    return f"- `{safe_text(item['id'])}`: `{safe_text(item['source'])}` {safe_text(item['kind'])} `{safe_text(item['target'])}`. {safe_text(item['explanation'])}"


def support_lines(catalog: dict[str, Any]) -> list[str]:
    lines = ["## Support and exclusions", ""]
    for item in catalog["support"]:
        detail = item.get("exclusion") or item.get("executable_proof")
        lines.append(
            f"- **{safe_text(item['label'])}** `{safe_text(item['id'])}`: {safe_text(detail)}"
        )
    return lines


def field_rows(model: dict[str, Any]) -> list[str]:
    rows = [
        "| Field | Absence | Description | Constraints |",
        "| --- | --- | --- | --- |",
    ]
    for field in model["fields"]:
        description = field.get("description") or "No separate catalog description."
        rows.append(
            f"| `{safe_text(field['name'])}` | `{safe_text(field['absence'])}` | {safe_text(description)} | `{_constraint(field['constraints'])}` |"
        )
    return rows


def component_lines(catalog: dict[str, Any], root: dict[str, Any]) -> list[str]:
    lines = ["## Component models", ""]
    for model in catalog["models"]:
        if model["id"] == root["id"]:
            continue
        lines += [
            f"### {safe_text(model['id'].removeprefix('model:'))}",
            "",
            safe_text(model["explanation"]),
            "",
            *field_rows(model),
            "",
        ]
    return lines


def render(catalog: dict[str, Any]) -> str:
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
        *support_lines(catalog),
        "",
        "## Fields and absence rules",
        "",
        *field_rows(root),
        "",
        "Every listed member follows its stated absence rule. Required nullable members distinguish an explicit null from a missing member. Rust validation remains authoritative.",
        "",
        *component_lines(catalog, root),
    ]
    lines += [
        "## Direct relationship diagram",
        "",
        "![ClinicalTrial fields connect to ten component models in adopted catalog order.](/downloads/biodata/clinical-trial-relationships.svg)",
        "",
        "[Download the accessible relationship diagram](/downloads/biodata/clinical-trial-relationships.svg).",
        "",
        "## Direct relationship text",
        "",
        "This text is equivalent to the diagram and remains available without images.",
        "",
    ]
    for item in direct_relationships(catalog):
        lines.append(relationship_line(item))
    lines += ["", "## Component relationships", ""]
    for item in catalog["relationships"]:
        lines.append(relationship_line(item))
    lines += [
        "",
        "## Provenance",
        "",
        f"This reference comes from catalog format {catalog['format_version']} and the recorded example `{safe_text(catalog['example_receipt']['fixture'])}`. {safe_text(catalog['example_receipt']['attribution'])}.",
        "",
        "## Reproduce the recorded ClinicalTrials.gov conversion",
        "",
        "The downloadable input is **transformed historical offline evidence**. It does not present current clinical information and it is not pristine live\\-provider bytes. Its recorded time describes capture rather than provider processing. Consult ClinicalTrials.gov for current information.",
        "",
        f"The [pinned fixture receipt](<{RECEIPT_URL}>) records `{safe_text(catalog['example_receipt']['transformation'])}` under `{safe_text(catalog['example_receipt']['license'])}` with attribution to {safe_text(catalog['example_receipt']['attribution'])}. The request was `{safe_text(catalog['example_receipt']['request'])}`.",
        "",
        "The receipt records capture at `2026-09-02T22:50:57.882811Z`. The provider processing date is not recorded. Transport decoding and removal of `/protocolSection/contactsLocationsModule/overallOfficials` produced this admitted fixture. ClinicalTrials.gov updates daily.",
        "",
        f"The recorded input SHA\\-256 is `{EXPECTED_INPUT_DIGEST}`. The adapter and catalog are pinned to BioData revision `{EXPECTED_REVISION}`.",
        "",
        "Run the complete offline example after normal locked dependency preparation:",
        "",
        "```console",
        "cargo run --locked --no-default-features --example biodata-clinical-trial-recorded -- --check",
        "```",
        "",
        "The example executes `ClinicalTrialsGovApiV2::parse`, `project_clinical_trial`, `into_digest_assertion`, and `Document::ClinicalTrialProjection`. It compares strict document encoding with the expected projection download. The projection preserves the trial identity, brief title, official title, source status, study type, phases, conditions, arms, interventions, planned outcome, sites, and other admitted fields. Its capture digest binds the converted result to the recorded bytes. Its conversion report states field preservation and conversion loss.",
        "",
        "## Reviewed crosswalks",
        "",
        "Crosswalk review and adapter support are separate facts. Rows without implementation evidence describe **correspondence only**, not executable conversion.",
        "",
        "| Crosswalk | External contract | Direction | Relationship | Local path | Qualification | Conversion |",
        "| --- | --- | --- | --- | --- | --- | --- |",
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
            f"| `{safe_text(item['id'])}` | [{safe_text(label)}](<{url}>) | `{safe_text(item['direction'])}` | `{safe_text(item['semantic_relationship'])}` | `{safe_text(local)}` | {safe_text(item['qualification'])} | {safe_text(conversion)} |"
        )
    lines += [
        "",
        "## Downloads",
        "",
        "- [ClinicalTrial schema](/downloads/biodata/clinical-trial.schema.json)",
        "- [ClinicalTrialProjection schema](/downloads/biodata/clinical-trial-projection.schema.json)",
        "- [ClinicalTrial relationship diagram](/downloads/biodata/clinical-trial-relationships.svg)",
        "- [Recorded transformed ClinicalTrials.gov input](/downloads/biodata/nct02576665-provider-types.json)",
        "- [Recorded ClinicalTrials.gov projection](/downloads/biodata/ctgov-clinical-trial-projection.json)",
        "- [Adopted catalog bundle](/downloads/biodata/clinical-trial-v1.bundle.json)",
        "- [ClinicalTrial discovery contract](/biodata/discovery/clinical-trial.json)",
        "",
        "Artifact digests from the adopted catalog:",
        "",
    ]
    for artifact in catalog["artifacts"]:
        lines.append(
            f"- `{safe_text(artifact['id'])}`: `{safe_text(artifact['sha256'])}`"
        )
    lines += [
        "",
        "## Scope",
        "",
        "This page renders only the claims and exclusions in the validated adopted catalog.",
        "",
    ]
    return "\n".join(lines)


def discovery_contract(catalog: dict[str, Any]) -> dict[str, Any]:
    routes = {
        "html": "https://biomcp.org/biodata/models/clinical-trial/",
        "raw_markdown": "https://biomcp.org/biodata/models/clinical-trial.md",
        "diagram": "https://biomcp.org/downloads/biodata/clinical-trial-relationships.svg",
        "recorded_input": "https://biomcp.org/downloads/biodata/nct02576665-provider-types.json",
        "expected_projection": "https://biomcp.org/downloads/biodata/ctgov-clinical-trial-projection.json",
        "schema": "https://biomcp.org/downloads/biodata/clinical-trial.schema.json",
        "projection_schema": "https://biomcp.org/downloads/biodata/clinical-trial-projection.schema.json",
        "catalog": "https://biomcp.org/downloads/biodata/clinical-trial-v1.bundle.json",
    }
    support = {item["id"]: item for item in catalog["support"]}
    conversion_support = []
    for support_id, authority_prefix in (
        ("support:fhir-r4-adapter", "FHIR"),
        ("support:cdisc-usdm-4-adapter", "CDISC USDM"),
    ):
        item = support[support_id]
        detail = item["exclusion"] or item["executable_proof"] or "No proof recorded."
        authorities = sorted(
            {
                crosswalk["external_authority"]
                for crosswalk in catalog["crosswalks"]
                if crosswalk["external_authority"].startswith(authority_prefix)
            }
        )
        qualifications = [
            crosswalk["qualification"]
            for crosswalk in catalog["crosswalks"]
            if crosswalk["external_authority"] in authorities
        ]
        conversion_support.append(
            f"{', '.join(authorities)}; {support_id}: {item['label']} ({detail}); "
            f"reviewed qualifications: {' '.join(qualifications)}"
        )
    return {
        "format_version": 1,
        "model": "model:ClinicalTrial",
        "routes": routes,
        "diagram_relationships": direct_relationships(catalog),
        "support": catalog["support"],
        "crosswalks": catalog["crosswalks"],
        "tasks": [
            {
                "question": "What fields describe a clinical trial?",
                "route": routes["html"],
                "answer": "The generated model reference lists every ClinicalTrial field and absence rule.",
            },
            {
                "question": "How is a clinical study related to its component records?",
                "route": routes["diagram"],
                "answer": "The diagram and adjacent text describe ten direct relationships; the model page describes all 20 catalog relationships.",
            },
            {
                "question": "How can I reproduce the recorded ClinicalTrials.gov conversion?",
                "route": routes["raw_markdown"],
                "answer": "Run the pinned offline Rust example against the recorded transformed input and compare its strict document with the expected projection.",
            },
            {
                "question": "Can BioData convert ClinicalTrial to FHIR or USDM?",
                "route": routes["html"],
                "answer": " ".join(conversion_support),
            },
        ],
    }
